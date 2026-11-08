#include "runtime_client.h"

#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QTimer>
#include <QUuid>
#include <utility>

namespace {
constexpr qsizetype frameLimit = 256 * 1024;
constexpr qsizetype pendingLimit = 32;
constexpr qsizetype writeLimit = 2 * frameLimit;
}

RuntimeClient::RuntimeClient(QString executable, QStringList arguments, QObject *parent)
    : QObject(parent), executable_(std::move(executable)), arguments_(std::move(arguments))
{
    process_.setProcessChannelMode(QProcess::SeparateChannels);
    process_.setStandardErrorFile(QProcess::nullDevice());
    connect(&process_, &QProcess::started, this, &RuntimeClient::ready);
    connect(&process_, &QProcess::readyReadStandardOutput, this, &RuntimeClient::receive);
    connect(&process_, &QProcess::finished, this, &RuntimeClient::processFinished);
    connect(&process_, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        if (error == QProcess::FailedToStart)
            rejectConnection(tr("The runtime could not start."));
        else if (error == QProcess::ReadError || error == QProcess::WriteError)
            rejectConnection(tr("The runtime transport failed."));
    });
}

RuntimeClient::~RuntimeClient()
{
    process_.disconnect(this);
    if (isRunning()) {
        process_.kill();
        process_.waitForFinished(1000);
    }
}

QString RuntimeClient::siblingExecutable()
{
#ifdef Q_OS_WIN
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("cgacp-runtime.exe"));
#else
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("cgacp-runtime"));
#endif
}

QString RuntimeClient::validateExecutable(const QString &path, bool requirePin)
{
    const QFileInfo info(path);
    if (!info.isAbsolute() || !info.isFile() || !info.isExecutable() || info.isSymLink() ||
        info.canonicalFilePath().isEmpty())
        return tr("The CGACP runtime executable is missing or invalid.");

    const QString pinPath = path + QStringLiteral(".sha256");
    const QFileInfo pinInfo(pinPath);
    if (!pinInfo.exists() && !pinInfo.isSymLink())
        return requirePin ? tr("The CGACP runtime checksum is missing.") : QString{};
    QFile pin(pinPath);
    if (pinInfo.isSymLink() || !pinInfo.isFile() || pinInfo.size() > 128 || !pin.open(QIODevice::ReadOnly))
        return tr("The CGACP runtime checksum is invalid.");

    QFile executable(info.canonicalFilePath());
    if (!executable.open(QIODevice::ReadOnly) || executable.size() > 128 * 1024 * 1024)
        return tr("The CGACP runtime could not be verified.");
    QCryptographicHash hash(QCryptographicHash::Sha256);
    if (!hash.addData(&executable) || hash.result().toHex() != pin.readAll().trimmed().toLower())
        return tr("The CGACP runtime checksum does not match.");
    return {};
}

bool RuntimeClient::isRunning() const
{
    return process_.state() != QProcess::NotRunning;
}

void RuntimeClient::start()
{
    if (isRunning())
        return;
    ++generation_;
    closing_ = false;
    disconnected_ = false;
    buffered_.clear();
    const bool isAppRuntime = executable_ == siblingExecutable();
    const auto invalid = validateExecutable(executable_, isAppRuntime);
    if (!invalid.isEmpty()) {
        rejectConnection(invalid);
        return;
    }
    const QFileInfo info(executable_);
    if (isAppRuntime && info.canonicalPath() != QFileInfo(QCoreApplication::applicationFilePath()).canonicalPath()) {
        rejectConnection(tr("The CGACP runtime is outside the application directory."));
        return;
    }
    process_.setWorkingDirectory(info.canonicalPath());
    process_.start(info.canonicalFilePath(), arguments_);
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
    process_.closeWriteChannel();
    const auto generation = generation_;
    QTimer::singleShot(1800, this, [this, generation] {
        if (generation == generation_ && isRunning())
            process_.terminate();
    });
    QTimer::singleShot(2600, this, [this, generation] {
        if (generation == generation_ && isRunning())
            process_.kill();
    });
}

void RuntimeClient::request(const QString &method, QJsonObject params, Reply reply)
{
    const QString id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const auto request = QJsonDocument(QJsonObject{
        {"v", 1}, {"id", id}, {"method", method}, {"params", std::move(params)}
    }).toJson(QJsonDocument::Compact) + '\n';
    QString error;
    if (process_.state() != QProcess::Running || closing_ || disconnected_)
        error = tr("The runtime is disconnected.");
    else if (pending_.size() >= pendingLimit || process_.bytesToWrite() > writeLimit)
        error = tr("The runtime request queue is full.");
    else if (request.size() > frameLimit)
        error = tr("The runtime request exceeds the size limit.");
    if (!error.isEmpty()) {
        if (reply)
            reply({}, error);
        return;
    }
    pending_.insert(id, std::move(reply));
    if (process_.write(request) != request.size()) {
        rejectConnection(tr("The runtime transport failed."));
        return;
    }
    QTimer::singleShot(15000, this, [this, id] {
        auto entry = pending_.find(id);
        if (entry == pending_.end())
            return;
        auto callback = std::move(entry.value());
        pending_.erase(entry);
        if (callback)
            callback({}, tr("The runtime request timed out. Check the saved state before retrying."));
    });
}

void RuntimeClient::receive()
{
    while (process_.bytesAvailable() > 0 && !disconnected_) {
        buffered_.append(process_.read(32768));
        while (true) {
            const auto newline = buffered_.indexOf('\n');
            if (newline < 0)
                break;
            if (newline > frameLimit) {
                rejectConnection(tr("The runtime response exceeds the size limit."));
                return;
            }
            const auto bytes = buffered_.left(newline);
            buffered_.remove(0, newline + 1);
            QJsonParseError parse;
            const auto document = QJsonDocument::fromJson(bytes, &parse);
            const auto response = document.object();
            const auto id = response.value("id").toString();
            const auto ok = response.value("ok");
            const auto result = response.value("result");
            const auto error = response.value("error");
            if (parse.error != QJsonParseError::NoError || !document.isObject() ||
                response.value("v").toInt() != 1 || id.isEmpty() || !ok.isBool() ||
                (ok.toBool() ? !result.isObject() : !error.isObject())) {
                rejectConnection(tr("The runtime sent an invalid response."));
                return;
            }
            auto entry = pending_.find(id);
            if (entry == pending_.end())
                continue;
            auto callback = std::move(entry.value());
            pending_.erase(entry);
            if (callback) {
                QString message;
                if (!ok.toBool()) {
                    const auto details = error.toObject();
                    message = details.value("message").toString().simplified().left(450);
                    if (message.isEmpty())
                        message = tr("The runtime rejected the operation.");
                }
                callback(result.toObject(), message);
            }
        }
        if (buffered_.size() > frameLimit) {
            rejectConnection(tr("The runtime response exceeds the size limit."));
            return;
        }
    }
}

void RuntimeClient::rejectConnection(const QString &reason)
{
    if (disconnected_)
        return;
    disconnected_ = true;
    buffered_.clear();
    failPending(reason);
    if (isRunning())
        process_.kill();
    if (!closing_)
        emit disconnected(reason);
}

void RuntimeClient::failPending(const QString &reason)
{
    auto queued = std::exchange(pending_, {});
    for (auto &reply : queued) {
        if (reply)
            reply({}, reason);
    }
}

void RuntimeClient::processFinished()
{
    if (closing_) {
        failPending(tr("The runtime stopped."));
        emit stopped();
    } else {
        rejectConnection(tr("The runtime stopped. Reconnect to restore saved state."));
    }
}
