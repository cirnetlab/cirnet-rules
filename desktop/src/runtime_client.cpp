#include "build_info.h"
#include "runtime_client.h"

#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDebug>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QPointer>
#include <QRegularExpression>
#include <QScopeGuard>
#include <QThread>
#include <QUuid>
#include <algorithm>
#include <utility>

namespace {
constexpr qsizetype frameLimit = 256 * 1024;
constexpr qsizetype pendingLimit = 32;
constexpr qsizetype viewReadReserve = 4;
constexpr qsizetype readsPerPurpose = 2;
constexpr qsizetype writeLimit = 2 * frameLimit;
constexpr qsizetype receiveLimit = (pendingLimit + viewReadReserve) * (frameLimit + 1);
constexpr int framesPerSlice = 4;
constexpr qint64 drainBudgetMs = 4;
constexpr qint64 requestTimeoutMs = 15000;
constexpr qsizetype traceLimit = 256 * 1024;

QByteArray traceVariable()
{
    return QByteArray(CIRNETRULES_PRODUCT_ID).toUpper() + "_TRACE_IPC";
}

QByteArray encodeRequest(const QString &id, const QString &method, QJsonObject params)
{
    return QJsonDocument(QJsonObject{
        {"v", CIRNETRULES_IPC_PROTOCOL}, {"id", id}, {"method", QStringLiteral(CIRNETRULES_API_NAMESPACE) + method},
        {"params", std::move(params)}
    }).toJson(QJsonDocument::Compact) + '\n';
}

QString requestId()
{
    return QUuid::createUuid().toString(QUuid::WithoutBraces);
}

QString checkExecutable(const QString &path, bool requirePin, const std::atomic_bool *cancelled = nullptr)
{
    const QFileInfo info(path);
    if (!info.isAbsolute() || !info.isFile() || !info.isExecutable() || info.isSymLink() ||
        info.canonicalFilePath().isEmpty())
        return RuntimeClient::tr("The local runtime executable is missing or invalid.");
    const QString pinPath = path + QStringLiteral(".sha256");
    const QFileInfo pinInfo(pinPath);
    if (!pinInfo.exists() && !pinInfo.isSymLink())
        return requirePin ? RuntimeClient::tr("The local runtime checksum is missing.") : QString{};
    QFile pin(pinPath);
    if (pinInfo.isSymLink() || !pinInfo.isFile() || pinInfo.size() > 128 || !pin.open(QIODevice::ReadOnly))
        return RuntimeClient::tr("The local runtime checksum is invalid.");
    QFile executable(info.canonicalFilePath());
    if (!executable.open(QIODevice::ReadOnly) || executable.size() > 128 * 1024 * 1024)
        return RuntimeClient::tr("The local runtime could not be verified.");
    QCryptographicHash hash(QCryptographicHash::Sha256);
    while (!executable.atEnd()) {
        if (cancelled && cancelled->load(std::memory_order_relaxed))
            return {};
        const auto chunk = executable.read(256 * 1024);
        if (chunk.isEmpty())
            return RuntimeClient::tr("The local runtime could not be verified.");
        hash.addData(chunk);
    }
    if (hash.result().toHex() != pin.readAll().trimmed().toLower())
        return RuntimeClient::tr("The local runtime checksum does not match.");
    return {};
}
}

RuntimeClient::RuntimeClient(QString executable, QStringList arguments, QObject *parent)
    : QObject(parent), executable_(std::move(executable)), arguments_(std::move(arguments)),
      process_(new QProcess(this))
{
    process_->setProcessChannelMode(QProcess::SeparateChannels);
    if (qgetenv(traceVariable().constData()) == "1") {
        traceBudget_ = traceLimit;
        connect(process_, &QProcess::readyReadStandardError, this, &RuntimeClient::readRuntimeTrace);
    } else {
        process_->setStandardErrorFile(QProcess::nullDevice());
    }
    expiry_.setInterval(250);
    connect(&expiry_, &QTimer::timeout, this, &RuntimeClient::expireRequests);
    readDispatch_.setSingleShot(true);
    connect(&readDispatch_, &QTimer::timeout, this, &RuntimeClient::dispatchReads);
    connect(process_, &QProcess::bytesWritten, this, [this] {
        if (hasQueuedRead() || std::ranges::any_of(pending_, [](const Pending &entry) {
                return entry.obsolete && !entry.cancelSent;
            }))
            scheduleReads();
    });
    connect(process_, &QProcess::started, this, [this] {
        request("runtime.info", {}, [this](const auto &info, const auto &error) {
            if (closing_ || disconnected_)
                return;
            if (!error.isEmpty() || !compatibleRuntime(info)) {
                rejectConnection(error.isEmpty() ? tr("Dashboard and runtime identities differ.") : error);
                return;
            }
            runtimeInfo_ = info;
            emit ready();
        });
    });
    connect(process_, &QProcess::readyReadStandardOutput, this, &RuntimeClient::receive);
    connect(process_, &QProcess::finished, this, &RuntimeClient::processFinished);
    connect(process_, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        if (error == QProcess::FailedToStart)
            rejectConnection(tr("The runtime could not start."));
        else if (error == QProcess::ReadError || error == QProcess::WriteError)
            rejectConnection(tr("The runtime transport failed."));
    });
}

RuntimeClient::~RuntimeClient()
{
    process_->disconnect(this);
    if (cancelValidation_)
        cancelValidation_->store(true, std::memory_order_relaxed);
    if (process_->state() == QProcess::NotRunning)
        return;
    // Let Qt reap an active child asynchronously when its view is destroyed.
    if (auto *application = QCoreApplication::instance(); application && !QCoreApplication::closingDown()) {
        process_->setParent(application);
        connect(process_, &QProcess::stateChanged, process_, [process = process_](QProcess::ProcessState state) {
            if (state == QProcess::NotRunning)
                process->deleteLater();
        });
    }
    process_->kill();
}

QString RuntimeClient::siblingExecutable()
{
#ifdef Q_OS_WIN
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral(CIRNETRULES_RUNTIME_BASENAME ".exe"));
#else
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral(CIRNETRULES_RUNTIME_BASENAME));
#endif
}

QString RuntimeClient::validateExecutable(const QString &path, bool requirePin)
{
    return checkExecutable(path, requirePin);
}

bool RuntimeClient::isRunning() const
{
    return validation_ || processExited_ || process_->state() != QProcess::NotRunning;
}

void RuntimeClient::start()
{
    if (isRunning())
        return;
    ++generation_;
    closing_ = false;
    disconnected_ = false;
    buffered_.clear();
    traceBuffer_.clear();
    runtimeInfo_ = {};
    drainScheduled_ = false;
    const bool isAppRuntime = executable_ == siblingExecutable();
    struct Validation { QString error, program, directory; };
    const auto outcome = std::make_shared<Validation>();
    cancelValidation_ = std::make_shared<std::atomic_bool>(false);
    validation_ = QThread::create([path = executable_, appPath = QCoreApplication::applicationFilePath(),
                                   isAppRuntime, outcome, stop = cancelValidation_] {
        outcome->error = checkExecutable(path, isAppRuntime, stop.get());
        if (!outcome->error.isEmpty() || stop->load(std::memory_order_relaxed))
            return;
        const QFileInfo info(path);
        outcome->program = info.canonicalFilePath();
        outcome->directory = info.canonicalPath();
        if (isAppRuntime && outcome->directory != QFileInfo(appPath).canonicalPath())
            outcome->error = RuntimeClient::tr("The local runtime is outside the application directory.");
    });
    connect(validation_, &QThread::finished, this, [this, outcome] {
        validation_ = nullptr;
        cancelValidation_.reset();
        if (closing_) {
            emit stopped();
            return;
        }
        if (!outcome->error.isEmpty()) {
            rejectConnection(outcome->error);
            return;
        }
        process_->setWorkingDirectory(outcome->directory);
        process_->start(outcome->program, arguments_);
    });
    connect(validation_, &QThread::finished, validation_, &QObject::deleteLater);
    validation_->start();
}

void RuntimeClient::stop()
{
    if (!isRunning()) {
        emit stopped();
        return;
    }
    if (closing_)
        return;
    closing_ = true;
    readDispatch_.stop();
    if (cancelValidation_) {
        cancelValidation_->store(true, std::memory_order_relaxed);
        return;
    }
    if (processExited_)
        return;
    process_->closeWriteChannel();
    const auto generation = generation_;
    QTimer::singleShot(1800, this, [this, generation] {
        if (generation == generation_ && isRunning())
            process_->terminate();
    });
    QTimer::singleShot(2600, this, [this, generation] {
        if (generation == generation_ && isRunning())
            process_->kill();
    });
}

void RuntimeClient::request(const QString &method, QJsonObject params, Reply reply)
{
    submit(method, std::move(params), Pending{std::move(reply), QDeadlineTimer(requestTimeoutMs), {}, {}});
}

void RuntimeClient::submit(const QString &method, QJsonObject params, Pending pending, qint64 queueUs)
{
    const auto id = requestId();
    const auto request = encodeRequest(id, method, std::move(params));
    const bool reserve = pending.purpose.has_value() || method == "runtime.info" || method == "operation.cancel";
    QString error;
    if (process_->state() != QProcess::Running || closing_ || disconnected_)
        error = tr("The runtime is disconnected.");
    else if (runtimeInfo_.isEmpty() && method != "runtime.info")
        error = tr("Wait for runtime identity verification before sending operations.");
    else if (request.size() > frameLimit)
        error = tr("The runtime request exceeds the size limit.");
    else if (pending_.size() >= (reserve ? pendingLimit : pendingLimit - viewReadReserve)
             || process_->bytesToWrite() + request.size() > writeLimit)
        error = tr("The runtime request queue is full.");
    if (!error.isEmpty()) {
        if (pending.callback)
            pending.callback({}, error);
        return;
    }
    auto entry = pending_.insert(id, std::move(pending));
    if (traceBudget_ > 0) {
        entry->submitted.start();
        traceLine("qt_ipc sent id=" + id.toLatin1() + " method=" + method.toLatin1()
                  + " bytes=" + QByteArray::number(request.size())
                  + " pending=" + QByteArray::number(pending_.size())
                  + " broker_us=" + QByteArray::number(queueUs));
    }
    if (!expiry_.isActive())
        expiry_.start();
    if (process_->write(request) != request.size()) {
        rejectConnection(tr("The runtime transport failed."));
        return;
    }
}

void RuntimeClient::latestRead(ViewRead purpose, QJsonObject params, Reply reply)
{
    if (closing_ || disconnected_ || runtimeInfo_.isEmpty()) {
        if (reply)
            reply({}, tr("The runtime is disconnected."));
        return;
    }
    ReadIntent intent{std::move(params), std::move(reply), QDeadlineTimer(requestTimeoutMs), {}};
    if (traceBudget_ > 0)
        intent.enqueued.start();
    replaceRead(purpose, std::move(intent));
}

void RuntimeClient::cancelLatestRead(ViewRead purpose)
{
    replaceRead(purpose, {});
}

void RuntimeClient::replaceRead(ViewRead purpose, std::optional<ReadIntent> next)
{
    auto &slot = queuedReads_[static_cast<size_t>(purpose)];
    auto previous = std::exchange(slot, std::move(next));
    QList<Reply> cancelled;
    if (previous)
        cancelled.append(std::move(previous->callback));
    for (auto &entry : pending_) {
        if (entry.purpose == purpose && !entry.obsolete) {
            entry.obsolete = true;
            cancelled.append(std::exchange(entry.callback, {}));
        }
    }
    if (slot || !cancelled.isEmpty())
        scheduleReads();
    if (slot && !expiry_.isActive())
        expiry_.start();
    if (traceBudget_ > 0 && !cancelled.isEmpty())
        traceLine("qt_ipc superseded=" + QByteArray::number(cancelled.size())
                  + " pending=" + QByteArray::number(pending_.size()));
    for (auto &callback : cancelled)
        if (callback)
            callback({{"code", "REQUEST_CANCELLED"}}, tr("The read request was superseded."));
}

bool RuntimeClient::hasQueuedRead() const
{
    return std::ranges::any_of(queuedReads_, [](const auto &slot) { return slot.has_value(); });
}

void RuntimeClient::scheduleReads()
{
    if (!closing_ && !disconnected_ && !readDispatch_.isActive())
        readDispatch_.start();
}

void RuntimeClient::dispatchReads()
{
    if (closing_ || disconnected_ || process_->state() != QProcess::Running)
        return;
    const QPointer<RuntimeClient> owner(this);
    if (runtimeInfo_.value("read_cancellation").toBool()) {
        for (auto entry = pending_.begin(); entry != pending_.end(); ++entry) {
            if (!entry->obsolete || entry->cancelSent)
                continue;
            const auto frame = encodeRequest(requestId(), QStringLiteral("request.cancel"), {{"id", entry.key()}});
            if (process_->bytesToWrite() + frame.size() > writeLimit)
                break;
            entry->cancelSent = true;
            if (process_->write(frame) != frame.size()) {
                rejectConnection(tr("The runtime transport failed."));
                return;
            }
        }
    }
    for (const auto purpose : {ViewRead::ConversationDetail, ViewRead::ConversationEvents}) {
        auto &slot = queuedReads_[static_cast<size_t>(purpose)];
        if (!slot || pending_.size() >= pendingLimit || process_->bytesToWrite() > writeLimit - frameLimit)
            continue;
        const auto outstanding = std::ranges::count_if(pending_, [purpose](const Pending &entry) {
            return entry.purpose == purpose;
        });
        // Retain credit for an obsolete frame until its reply or deadline. Keep
        // one extra credit for the new selection while cancellation is pending.
        if (outstanding >= readsPerPurpose)
            continue;
        auto intent = std::exchange(slot, {});
        if (intent->deadline.hasExpired()) {
            if (intent->callback)
                intent->callback({}, tr("The runtime read timed out."));
            if (!owner)
                return;
            continue;
        }
        const auto method = purpose == ViewRead::ConversationDetail
            ? QStringLiteral("conversation.get") : QStringLiteral("conversation.events");
        const auto queued = intent->enqueued.isValid() ? intent->enqueued.nsecsElapsed() / 1000 : 0;
        submit(method, std::move(intent->params),
               Pending{std::move(intent->callback), intent->deadline, {}, purpose}, queued);
        if (!owner)
            return;
    }
}

void RuntimeClient::receive()
{
    if (process_->bytesAvailable() + buffered_.size() > receiveLimit) {
        rejectConnection(tr("The runtime response queue exceeds the size limit."));
        return;
    }
    if (drainScheduled_ || draining_)
        return;
    draining_ = true;
    const QPointer<RuntimeClient> owner(this);
    const auto guard = qScopeGuard([owner] {
        if (owner)
            owner->draining_ = false;
    });
    QElapsedTimer elapsed;
    elapsed.start();
    int frames = 0;
    while (!disconnected_ && frames < framesPerSlice && elapsed.elapsed() < drainBudgetMs) {
        auto newline = buffered_.indexOf('\n');
        if (newline < 0 && process_->bytesAvailable() > 0) {
            buffered_.append(process_->read(32768));
            newline = buffered_.indexOf('\n');
        }
        if (newline < 0) {
            if (buffered_.size() > frameLimit) {
                rejectConnection(tr("The runtime response exceeds the size limit."));
                return;
            }
            if (process_->bytesAvailable() == 0)
                break;
            continue;
        }
        ++frames;
        if (newline > frameLimit) {
            rejectConnection(tr("The runtime response exceeds the size limit."));
            return;
        }
        const auto bytes = buffered_.left(newline);
        buffered_.remove(0, newline + 1);
        const auto parseStart = elapsed.nsecsElapsed();
        QJsonParseError parse;
        const auto document = QJsonDocument::fromJson(bytes, &parse);
        const auto response = document.object();
        const auto id = response.value("id").toString();
        const auto ok = response.value("ok");
        const auto result = response.value("result");
        const auto error = response.value("error");
        if (parse.error == QJsonParseError::NoError && document.isObject() &&
            response.value("v").isDouble() && response.value("v").toInt() != CIRNETRULES_IPC_PROTOCOL) {
            rejectConnection(tr("The runtime protocol is incompatible. Use the runtime installed with %1.").arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME)));
            return;
        }
        if (parse.error != QJsonParseError::NoError || !document.isObject() ||
            response.value("v").toInt() != CIRNETRULES_IPC_PROTOCOL || id.isEmpty() || !ok.isBool() ||
            (ok.toBool() ? !result.isObject() : !error.isObject())) {
            rejectConnection(tr("The runtime sent an invalid response."));
            return;
        }
        const auto parseUs = (elapsed.nsecsElapsed() - parseStart) / 1000;
        auto entry = pending_.find(id);
        if (entry == pending_.end())
            continue;
        auto callback = std::move(entry.value().callback);
        const auto submitted = entry->submitted;
        pending_.erase(entry);
        if (pending_.isEmpty() && !hasQueuedRead())
            expiry_.stop();
        if (hasQueuedRead())
            scheduleReads();
        if (callback) {
            QString message;
            auto payload = result.toObject();
            if (!ok.toBool()) {
                const auto details = error.toObject();
                payload = details.value("details").toObject();
                payload.insert("code", details.value("code"));
                message = details.value("message").toString().simplified().left(450);
                if (message.isEmpty())
                    message = tr("The runtime rejected the operation.");
            }
            const auto applyStart = elapsed.nsecsElapsed();
            const auto roundTripUs = submitted.isValid() ? submitted.nsecsElapsed() / 1000 : 0;
            callback(payload, message);
            if (!owner)
                return;
            if (submitted.isValid() && traceBudget_ > 0)
                traceLine("qt_ipc reply id=" + id.toLatin1()
                          + " roundtrip_us=" + QByteArray::number(roundTripUs)
                          + " parse_us=" + QByteArray::number(parseUs)
                          + " callback_us=" + QByteArray::number((elapsed.nsecsElapsed() - applyStart) / 1000)
                          + " bytes=" + QByteArray::number(bytes.size()));
        }
    }
    if (!disconnected_ && (process_->bytesAvailable() > 0 || buffered_.contains('\n'))) {
        drainScheduled_ = true;
        const auto generation = generation_;
        QTimer::singleShot(0, this, [this, generation] {
            if (generation == generation_) {
                drainScheduled_ = false;
                receive();
            }
        });
    } else if (processExited_) {
        finishProcess();
    }
}

void RuntimeClient::traceLine(QByteArray line)
{
    if (line.size() + 1 > traceBudget_) {
        traceBudget_ = 0;
        return;
    }
    traceBudget_ -= line.size() + 1;
    qInfo().noquote() << QString::fromLatin1(line);
}

void RuntimeClient::readRuntimeTrace()
{
    const auto chunk = process_->readAllStandardError();
    if (traceBudget_ == 0)
        return;
    // Relay only correlated numeric records. Runtime diagnostics may contain paths.
    static const QRegularExpression record(
        QStringLiteral("^cirnetrules-ipc id=[a-f0-9-]{36} method=[a-z_.]{1,64}"
                       "(?: [a-z_]{1,32}=[0-9]{1,20}){1,12}$"));
    traceBuffer_.append(chunk);
    while (traceBudget_ > 0) {
        const auto end = traceBuffer_.indexOf('\n');
        if (end < 0)
            break;
        const auto line = traceBuffer_.left(end).trimmed();
        traceBuffer_.remove(0, end + 1);
        if (line.size() <= 512 && record.match(QString::fromLatin1(line)).hasMatch())
            traceLine(line);
    }
    if (traceBuffer_.size() > 512 || traceBudget_ == 0)
        traceBuffer_.clear();
}

void RuntimeClient::expireRequests()
{
    QList<Reply> expired;
    for (auto it = pending_.begin(); it != pending_.end();) {
        if (it.value().deadline.hasExpired()) {
            expired.append(std::move(it.value().callback));
            it = pending_.erase(it);
        } else {
            ++it;
        }
    }
    for (auto &slot : queuedReads_) {
        if (slot && slot->deadline.hasExpired()) {
            expired.append(std::move(slot->callback));
            slot.reset();
        }
    }
    if (hasQueuedRead())
        scheduleReads();
    else if (pending_.isEmpty())
        expiry_.stop();
    for (auto &callback : expired)
        if (callback)
            callback({}, tr("The runtime request timed out. Check the saved state before retrying."));
}

void RuntimeClient::rejectConnection(const QString &reason)
{
    if (disconnected_)
        return;
    disconnected_ = true;
    buffered_.clear();
    const QPointer<RuntimeClient> owner(this);
    failPending(reason);
    if (!owner)
        return;
    if (process_->state() != QProcess::NotRunning)
        process_->kill();
    if (!closing_)
        emit disconnected(reason);
    if (owner && processExited_)
        finishProcess();
}

void RuntimeClient::failPending(const QString &reason)
{
    auto queued = std::exchange(pending_, {});
    auto reads = std::exchange(queuedReads_, {});
    expiry_.stop();
    readDispatch_.stop();
    for (auto &reply : queued) {
        if (reply.callback)
            reply.callback({}, reason);
    }
    for (auto &slot : reads)
        if (slot && slot->callback)
            slot->callback({}, reason);
}

void RuntimeClient::processFinished()
{
    processExited_ = true;
    receive();
}

void RuntimeClient::finishProcess()
{
    processExited_ = false;
    if (closing_) {
        const QPointer<RuntimeClient> owner(this);
        failPending(tr("The runtime stopped."));
        if (owner)
            emit stopped();
    } else {
        rejectConnection(tr("The runtime stopped. Reconnect to restore saved state."));
    }
}
