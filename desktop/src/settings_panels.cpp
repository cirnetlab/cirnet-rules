#include "settings_panels.h"

#include <QApplication>
#include <QClipboard>
#include <QComboBox>
#include <QDateTime>
#include <QDesktopServices>
#include <QDir>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QLocale>
#include <QMessageBox>
#include <QPainter>
#include <QPointer>
#include <QProgressBar>
#include <QPushButton>
#include <QSignalBlocker>
#include <QTimer>
#include <QUrl>
#include <QVBoxLayout>

#include <array>
#include <utility>

namespace {
QLabel *textLabel(const QString &value, const char *role = "meta")
{
    auto *label = new QLabel(value);
    label->setObjectName(QLatin1StringView(role));
    label->setTextFormat(Qt::PlainText);
    label->setWordWrap(true);
    return label;
}

QPushButton *button(const QString &text, const char *name, QBoxLayout &layout)
{
    auto *control = new QPushButton(text);
    control->setObjectName(QLatin1StringView(name));
    layout.addWidget(control);
    return control;
}

QVBoxLayout *group(QVBoxLayout &parent, const QString &heading)
{
    auto *frame = new QFrame;
    frame->setObjectName("settingsGroup");
    auto *body = new QVBoxLayout(frame);
    body->addWidget(textLabel(heading, "sectionTitle"));
    parent.addWidget(frame);
    return body;
}

QString sizeText(qint64 bytes)
{
    return QLocale().formattedDataSize(bytes, 2, QLocale::DataSizeIecFormat);
}

QString dateText(qint64 epoch)
{
    return epoch > 0 ? QLocale().toString(QDateTime::fromSecsSinceEpoch(epoch).toLocalTime(), QLocale::ShortFormat)
                     : QString{};
}

class StorageBar final : public QWidget {
public:
    StorageBar() { setFixedHeight(16); }
    void setParts(std::array<qint64, 3> parts) { parts_ = parts; update(); }

protected:
    void paintEvent(QPaintEvent *) override
    {
        QPainter painter(this);
        painter.fillRect(rect(), palette().color(QPalette::Midlight));
        const qreal total = qreal(parts_[0]) + qreal(parts_[1]) + qreal(parts_[2]);
        if (total <= 0)
            return;
        const std::array<QColor, 3> colors{
            palette().color(QPalette::Highlight), QColor("#6695d5"), QColor("#9b8bd3")
        };
        qreal x = 0;
        for (size_t index = 0; index < parts_.size(); ++index) {
            const qreal partWidth = width() * qreal(parts_[index]) / total;
            painter.fillRect(QRectF(x, 0, partWidth, height()), colors[index]);
            x += partWidth;
        }
    }

private:
    std::array<qint64, 3> parts_{};
};

class StoragePage final : public QWidget {
    Q_DECLARE_TR_FUNCTIONS(StoragePage)

public:
    StoragePage(RuntimeClient &client, SettingsRequest request, std::function<void()> openArchived)
        : request_(std::move(request))
    {
        setObjectName("storagePage");
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(textLabel(tr("Storage"), "heading"));
        auto *device = group(*layout, tr("This device"));
        accounting_ = textLabel(tr("Measure the CGACP data directory to see its current size."));
        accounting_->setObjectName("storageAccounting");
        accounting_->setTextInteractionFlags(Qt::TextSelectableByMouse);
        device->addWidget(accounting_);
        bar_ = new StorageBar;
        bar_->setAccessibleName(tr("Physical storage breakdown"));
        bar_->hide();
        device->addWidget(bar_);
        path_ = textLabel({});
        path_->setObjectName("storageDataRoot");
        path_->setTextInteractionFlags(Qt::TextSelectableByMouse);
        device->addWidget(path_);
        auto *scanRow = new QHBoxLayout;
        scan_ = button(tr("Measure storage"), "scanStorage", *scanRow);
        scanRow->addStretch();
        device->addLayout(scanRow);
        scanRow = new QHBoxLayout;
        copy_ = button(tr("Copy data path"), "copyStorageRoot", *scanRow);
        open_ = button(tr("Open data folder"), "openStorageRoot", *scanRow);
        scanRow->addStretch();
        device->addLayout(scanRow);
        auto *backupGroup = group(*layout, tr("Backups and database maintenance"));
        backups_ = new QComboBox;
        backups_->setObjectName("storageBackups");
        backups_->setAccessibleName(tr("Database or backup to verify"));
        backups_->addItem(tr("Live database"));
        backupGroup->addWidget(backups_);
        auto *backupRow = new QHBoxLayout;
        backup_ = button(tr("Create backup"), "createStorageBackup", *backupRow);
        verify_ = button(tr("Verify selected"), "verifyStorageBackup", *backupRow);
        backupRow->addStretch();
        backupGroup->addLayout(backupRow);
        backupRow = new QHBoxLayout;
        compact_ = button(tr("Review compaction"), "compactStorage", *backupRow);
        backupRow->addStretch();
        backupGroup->addLayout(backupRow);
        backupGroup->addWidget(textLabel(tr("Backups may contain earlier chat content. Verification does not restore or overwrite your data.")));
        auto *cleanupGroup = group(*layout, tr("Review stored chats"));
        auto *archived = button(tr("Manage archived chats"), "storageArchivedChats", *cleanupGroup);
        cleanupGroup->addWidget(textLabel(tr("Worktree, cache, and log removal is unavailable without recorded ownership. Registered repositories are kept.")));
        auto *operationRow = new QHBoxLayout;
        progress_ = new QProgressBar;
        progress_->setObjectName("storageProgress");
        progress_->setAccessibleName(tr("Storage operation progress"));
        progress_->hide();
        operationRow->addWidget(progress_, 1);
        cancel_ = button(tr("Cancel operation"), "cancelStorageOperation", *operationRow);
        retry_ = button(tr("Retry status"), "retryStorageOperation", *operationRow);
        layout->addLayout(operationRow);
        status_ = textLabel({});
        status_->setObjectName("storageOperationStatus");
        layout->addWidget(status_);
        layout->addStretch();

        poll_.setSingleShot(true);
        poll_.setInterval(250);
        connect(&poll_, &QTimer::timeout, this, [this] { poll(false); });
        connect(&client, &RuntimeClient::disconnected, this, [this] {
            ++generation_;
            poll_.stop();
            if (pending_ || !operation_.isEmpty())
                status_->setText(tr("The runtime disconnected. Measure storage again to check the outcome."));
            operation_.clear();
            pending_ = false;
            statusFailed_ = false;
            cancellationQueued_ = false;
            cancellationInFlight_ = false;
            cancelling_ = false;
            freePages_ = 0;
            syncActions();
        });
        connect(scan_, &QPushButton::clicked, this, [this] { start("storage.scan"); });
        connect(backup_, &QPushButton::clicked, this, [this] { start("storage.backup.start"); });
        connect(verify_, &QPushButton::clicked, this, [this] {
            const auto name = backups_->currentData().toString();
            start("storage.verify.start", name.isEmpty() ? QJsonObject{} : QJsonObject{{"backup", name}});
        });
        connect(compact_, &QPushButton::clicked, this, [this] { previewCompaction(); });
        connect(cancel_, &QPushButton::clicked, this, [this] {
            cancellationQueued_ = true;
            if (!pending_)
                poll(true);
            syncActions();
        });
        connect(retry_, &QPushButton::clicked, this, [this] { poll(cancellationQueued_); });
        connect(copy_, &QPushButton::clicked, this, [this] { QApplication::clipboard()->setText(path_->text()); });
        connect(open_, &QPushButton::clicked, this, [this] {
            if (QDir::isAbsolutePath(path_->text()) &&
                !QDesktopServices::openUrl(QUrl::fromLocalFile(path_->text())))
                status_->setText(tr("Could not open the data folder."));
        });
        connect(archived, &QPushButton::clicked, this, std::move(openArchived));
        syncActions();
    }

protected:
    void showEvent(QShowEvent *event) override
    {
        QWidget::showEvent(event);
        if (!operation_.isEmpty() && !pending_ && !statusFailed_)
            poll_.start();
    }
    void hideEvent(QHideEvent *event) override
    {
        poll_.stop();
        QWidget::hideEvent(event);
    }

private:
    void call(const QString &method, QJsonObject params)
    {
        pending_ = true;
        syncActions();
        request_(method, std::move(params), [owner = QPointer(this), generation = generation_](const auto &reply, const auto &error) {
            if (owner && owner->generation_ == generation)
                owner->receive(reply, error);
        });
    }

    void start(const QString &method, QJsonObject params = {})
    {
        if (pending_ || !operation_.isEmpty())
            return;
        status_->setText(tr("Submitting storage operation…"));
        progress_->setRange(0, 0);
        statusFailed_ = false;
        call(method, std::move(params));
    }

    void poll(bool cancel)
    {
        if (operation_.isEmpty() || pending_)
            return;
        poll_.stop();
        cancellationQueued_ = false;
        cancellationInFlight_ = cancel;
        statusFailed_ = false;
        call(cancel ? "operation.cancel" : "operation.get", {{"operation_id", operation_}});
    }

    void receive(const QJsonObject &reply, const QString &error)
    {
        pending_ = false;
        const auto id = reply.value("operation_id").toString();
        if (!error.isEmpty() || id.isEmpty() || (!operation_.isEmpty() && id != operation_)) {
            cancellationQueued_ |= std::exchange(cancellationInFlight_, false);
            statusFailed_ = true;
            status_->setText(error.isEmpty() ? tr("Storage operation status is unavailable.") : error);
            syncActions();
            return;
        }
        operation_ = id;
        const auto state = reply.value("state").toString();
        cancellationInFlight_ = false;
        cancelling_ = state == "CANCEL_REQUESTED";
        const auto progress = reply.value("progress").toObject();
        const auto phase = progress.value("phase").toString();
        const auto total = progress.value("total").toInteger();
        const auto completed = progress.value("completed").toInteger();
        progress_->setRange(0, total > 0 ? 100 : 0);
        if (total > 0)
            progress_->setValue(qBound(0, int(100.0 * qreal(completed) / qreal(total)), 100));
        if (state == "SUCCEEDED") {
            const auto kind = reply.value("kind").toString();
            const auto result = reply.value("result").toObject();
            if (kind == "SCAN")
                renderScan(result);
            else if (kind == "COMPACT")
                freePages_ = 0;  // A fresh measurement is required for another confirmation.
            const auto backup = result.value("backup").toString();
            QString summary = tr("Storage operation completed.");
            if (!backup.isEmpty()) {
                if (backups_->findData(backup) < 0)
                    backups_->addItem(backup, backup);
                backups_->setCurrentIndex(backups_->findData(backup));
                summary = tr("Storage operation completed. Backup: %1").arg(backup);
            } else if (kind == "COMPACT") {
                summary = tr("Storage operation completed. Database pages: %1 before, %2 after.")
                    .arg(result.value("before_page_count").toInteger()).arg(result.value("after_page_count").toInteger());
            }
            status_->setText(summary);
            operation_.clear();
        } else if (state == "FAILED" || state == "CANCELLED") {
            const auto issue = reply.value("error").toObject();
            status_->setText(state == "CANCELLED" ? tr("Storage operation cancelled.")
                                                  : tr("Storage operation failed: %1").arg(issue.value("message").toString()));
            operation_.clear();
        } else if (state == "CANCEL_REQUESTED") {
            cancellationQueued_ = false;
            status_->setText(tr("Waiting for the storage worker to stop…"));
        } else {
            status_->setText(tr("Storage operation: %1").arg(phase.isEmpty() ? state : phase));
            if (cancellationQueued_) {
                poll(true);
                return;
            }
        }
        if (operation_.isEmpty())
            cancellationQueued_ = false;
        else if (isVisible())
            poll_.start();
        syncActions();
    }

    void renderScan(const QJsonObject &snapshot)
    {
        const auto physical = snapshot.value("physical").toObject();
        const auto logical = snapshot.value("logical").toObject();
        const auto db = physical.value("database_bytes").toInteger();
        const auto sidecars = physical.value("wal_bytes").toInteger() + physical.value("shm_bytes").toInteger()
                            + physical.value("journal_bytes").toInteger();
        const auto backup = physical.value("backup_bytes").toInteger() + physical.value("pending_backup_bytes").toInteger();
        pageSize_ = logical.value("page_size").toInteger();
        pageCount_ = logical.value("page_count").toInteger();
        freePages_ = logical.value("free_pages").toInteger();
        const auto partial = snapshot.value("partial").toBool();
        const auto physicalLine = tr("%1 on disk · Database %2 · Sidecars %3 · Backups %4")
                                  .arg(sizeText(physical.value("total_bytes").toInteger()), sizeText(db),
                                       sizeText(sidecars), sizeText(backup));
        const auto logicalLine = tr("Logical message content: %1 active, %2 archived. Reusable database pages: %3.")
                                 .arg(sizeText(logical.value("active_message_bytes").toInteger()),
                                      sizeText(logical.value("archived_message_bytes").toInteger()),
                                      sizeText(pageSize_ * freePages_));
        const auto measured = tr("Measured %1").arg(dateText(snapshot.value("measured_at").toInteger()));
        const auto coverage = partial ? tr("Partial measurement. Skipped entries: %1.").arg(snapshot.value("skipped").toInt())
                                      : QString{};
        accounting_->setText(QStringList{physicalLine, logicalLine, measured, coverage}.join('\n').trimmed());
        path_->setText(snapshot.value("data_root").toString());
        bar_->setParts({db, sidecars, backup});
        bar_->show();
        const auto selected = backups_->currentData().toString();
        backups_->clear();
        backups_->addItem(tr("Live database"));
        for (const auto &value : snapshot.value("backups").toArray()) {
            const auto item = value.toObject();
            const auto name = item.value("name").toString();
            if (!name.isEmpty())
                backups_->addItem(QStringLiteral("%1 · %2").arg(name, sizeText(item.value("bytes").toInteger())), name);
        }
        if (const auto index = backups_->findData(selected); index >= 0)
            backups_->setCurrentIndex(index);
    }

    void previewCompaction()
    {
        const auto pages = pageCount_;
        const auto free = freePages_;
        auto *dialog = new QMessageBox(QMessageBox::Question, tr("Compact local database"),
            tr("The last measurement found %1 in reusable database pages. Compaction may reclaim some space and can take time.")
              .arg(sizeText(pageSize_ * free)), QMessageBox::Yes | QMessageBox::Cancel, this);
        dialog->setTextFormat(Qt::PlainText);
        dialog->setAttribute(Qt::WA_DeleteOnClose);
        dialog->setDefaultButton(QMessageBox::Cancel);
        connect(dialog, &QMessageBox::finished, this, [this, pages, free](int answer) {
            if (answer == QMessageBox::Yes)
                start("storage.compact.start", {{"expected_page_count", pages}, {"expected_free_pages", free}});
        });
        dialog->open();
    }

    void syncActions()
    {
        const bool idle = !pending_ && operation_.isEmpty();
        for (auto *control : {scan_, backup_, verify_})
            control->setEnabled(idle);
        compact_->setEnabled(idle && freePages_ > 0);
        const auto hasPath = QDir::isAbsolutePath(path_->text());
        copy_->setEnabled(hasPath);
        open_->setEnabled(hasPath);
        progress_->setVisible(!operation_.isEmpty() || pending_);
        cancel_->setEnabled(!operation_.isEmpty() && !cancellationQueued_ && !cancellationInFlight_ &&
                            !cancelling_ && !statusFailed_);
        retry_->setVisible(!operation_.isEmpty() && statusFailed_);
    }

    SettingsRequest request_;
    QTimer poll_;
    quint64 generation_ = 0;
    QString operation_;
    bool pending_ = false;
    bool cancellationQueued_ = false;
    bool cancellationInFlight_ = false;
    bool cancelling_ = false;
    bool statusFailed_ = false;
    qint64 pageSize_ = 0, pageCount_ = 0, freePages_ = 0;
    QLabel *accounting_ = nullptr, *path_ = nullptr, *status_ = nullptr;
    StorageBar *bar_ = nullptr;
    QProgressBar *progress_ = nullptr;
    QComboBox *backups_ = nullptr;
    QPushButton *scan_ = nullptr, *copy_ = nullptr, *open_ = nullptr, *backup_ = nullptr,
                *verify_ = nullptr, *compact_ = nullptr, *cancel_ = nullptr, *retry_ = nullptr;
};

class ArchivedPage final : public QWidget {
    Q_DECLARE_TR_FUNCTIONS(ArchivedPage)

public:
    ArchivedPage(SettingsRequest request, std::function<QJsonArray()> bindings, std::function<void()> changed,
                 DeleteChat deleteChat)
        : request_(std::move(request)), bindings_(std::move(bindings)), changed_(std::move(changed)),
          deleteChat_(std::move(deleteChat))
    {
        setObjectName("archivedChatsPage");
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(textLabel(tr("Archived chats"), "heading"));
        search_ = new QLineEdit;
        search_->setObjectName("archivedChatSearch");
        search_->setProperty("searchField", true);
        search_->setClearButtonEnabled(true);
        search_->setMaxLength(120);
        search_->setPlaceholderText(tr("Search archived chats"));
        search_->setAccessibleName(tr("Search archived chats"));
        layout->addWidget(search_);
        auto *filters = new QHBoxLayout;
        repository_ = new QComboBox;
        repository_->setObjectName("archivedRepositoryFilter");
        repository_->setAccessibleName(tr("Repository filter"));
        repository_->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
        repository_->setMinimumContentsLength(12);
        date_ = new QComboBox;
        date_->setObjectName("archivedDateFilter");
        date_->setAccessibleName(tr("Archive date filter"));
        date_->addItem(tr("Any date"), 0);
        date_->addItem(tr("Older than 30 days"), 30);
        date_->addItem(tr("Older than 90 days"), 90);
        filters->addWidget(repository_, 1);
        filters->addWidget(date_);
        layout->addLayout(filters);
        auto *sources = new QHBoxLayout;
        source_ = new QComboBox;
        source_->setObjectName("archivedSourceFilter");
        source_->setAccessibleName(tr("Chat source filter"));
        source_->addItem(tr("Any source"));
        source_->addItem(tr("Desktop"), "DASHBOARD");
        source_->addItem(tr("Automation"), "AUTOMATION");
        sources->addWidget(source_);
        sources->addStretch();
        layout->addLayout(sources);
        state_ = textLabel({});
        state_->setObjectName("archivedChatsState");
        layout->addWidget(state_);
        chats_ = new QListWidget;
        chats_->setObjectName("archivedChats");
        chats_->setAccessibleName(tr("Archived chats"));
        chats_->setWordWrap(true);
        chats_->setTextElideMode(Qt::ElideRight);
        layout->addWidget(chats_, 1);
        auto *paging = new QHBoxLayout;
        first_ = button(tr("First page"), "archivedFirstPage", *paging);
        next_ = button(tr("Next page"), "archivedNextPage", *paging);
        reload_ = button(tr("Refresh"), "archivedRefresh", *paging);
        paging->addStretch();
        layout->addLayout(paging);
        auto *actions = new QHBoxLayout;
        unarchive_ = button(tr("Unarchive"), "unarchiveChat", *actions);
        erase_ = button(tr("Review deletion"), "deleteArchivedChat", *actions);
        actions->addStretch();
        layout->addLayout(actions);

        debounce_.setSingleShot(true);
        debounce_.setInterval(220);
        connect(&debounce_, &QTimer::timeout, this, [this] { load(); });
        connect(search_, &QLineEdit::textChanged, this, [this] { resetQuery(true); });
        connect(repository_, &QComboBox::currentIndexChanged, this, [this] { resetQuery(false); });
        connect(date_, &QComboBox::currentIndexChanged, this, [this] { resetQuery(false); });
        connect(source_, &QComboBox::currentIndexChanged, this, [this] { resetQuery(false); });
        connect(first_, &QPushButton::clicked, this, [this] { resetQuery(false); });
        connect(reload_, &QPushButton::clicked, this, [this] { load(); });
        connect(next_, &QPushButton::clicked, this, [this] {
            before_ = nextBefore_;
            load();
        });
        connect(chats_, &QListWidget::currentItemChanged, this, [this] { syncActions(); });
        connect(unarchive_, &QPushButton::clicked, this, [this] { unarchive(); });
        connect(erase_, &QPushButton::clicked, this, [this] {
            mutation_ = true;
            syncActions();
            deleteChat_(selection().value("id").toString(), [owner = QPointer(this)](bool success) {
                if (!owner)
                    return;
                owner->mutation_ = false;
                if (success)
                    owner->resetQuery(false);
                else
                    owner->syncActions();
            });
        });
        syncActions();
    }

protected:
    void showEvent(QShowEvent *event) override
    {
        QWidget::showEvent(event);
        const QSignalBlocker blocked(repository_);
        const auto previous = repository_->currentData().toString();
        repository_->clear();
        repository_->addItem(tr("All repositories"));
        for (const auto &value : bindings_()) {
            const auto binding = value.toObject();
            const auto id = binding.value("binding_id").toString();
            const auto name = binding.value("display_name").toString();
            if (!id.isEmpty())
                repository_->addItem(name.isEmpty() ? id : name, id);
        }
        if (const int index = repository_->findData(previous); index >= 0)
            repository_->setCurrentIndex(index);
        if (previous != repository_->currentData().toString())
            resetQuery(false);
        else
            load();
    }
    void hideEvent(QHideEvent *event) override
    {
        debounce_.stop();
        QWidget::hideEvent(event);
    }

private:
    static bool hasCursor(const QJsonValue &value) { return !value.isNull() && !value.isUndefined(); }

    QJsonObject selection() const
    {
        const auto *item = chats_->currentItem();
        return item ? item->data(Qt::UserRole).toJsonObject() : QJsonObject{};
    }

    void call(const QString &method, QJsonObject params, RuntimeClient::Reply callback)
    {
        request_(method, std::move(params), [owner = QPointer(this), callback = std::move(callback)]
                 (const auto &reply, const auto &error) {
            if (owner)
                callback(reply, error);
        });
    }

    void resetQuery(bool debounce)
    {
        ++generation_;
        before_ = {};
        nextBefore_ = {};
        const auto days = date_->currentData().toInt();
        cutoff_ = days ? QDateTime::currentSecsSinceEpoch() - qint64(days) * 86400 : 0;
        queryReady_ = false;
        if (debounce) {
            debounce_.start();
            syncActions();
        } else {
            debounce_.stop();
            load();
        }
    }

    void load()
    {
        if (!isVisible() || loading_ || mutation_)
            return;
        const auto query = search_->text().trimmed();
        if (query.toUtf8().size() > 160) {
            state_->setText(tr("Shorten the search to 160 UTF-8 bytes."));
            syncActions();
            return;
        }
        loading_ = true;
        queryReady_ = false;
        state_->setText(tr("Loading archived chats…"));
        syncActions();
        QJsonObject params{{"state", "ARCHIVED"}, {"limit", 30}, {"query", query}};
        if (hasCursor(before_))
            params.insert("before", before_);
        if (const auto binding = repository_->currentData().toString(); !binding.isEmpty())
            params.insert("binding_id", binding);
        if (const auto source = source_->currentData().toString(); !source.isEmpty())
            params.insert("source", source);
        if (cutoff_ > 0)
            params.insert("updated_before", cutoff_);
        const auto generation = generation_;
        call("conversation.list", params, [this, generation](const auto &result, const auto &error) {
            loading_ = false;
            if (generation != generation_) {
                if (!debounce_.isActive())
                    load();
                return;
            }
            if (!error.isEmpty() || !result.value("items").isArray()) {
                state_->setText(error.isEmpty() ? tr("Could not load archived chats.")
                                                : tr("Could not load archived chats: %1").arg(error));
                reload_->setText(tr("Retry"));
                syncActions();
                return;
            }
            const auto selected = selection().value("id").toString();
            const auto items = result.value("items").toArray();
            nextBefore_ = result.value("next_before");
            chats_->clear();
            for (const auto &value : items) {
                const auto chat = value.toObject();
                const auto title = chat.value("title").toString();
                const auto timestamp = dateText(chat.value("updated_at").toInteger());
                auto *item = new QListWidgetItem(timestamp.isEmpty() ? title : title + '\n' + timestamp, chats_);
                item->setData(Qt::UserRole, chat);
                if (chat.value("id").toString() == selected)
                    chats_->setCurrentItem(item);
            }
            const auto total = result.value("total").toInteger(-1);
            if (items.isEmpty())
                state_->setText(total == 0 ? tr("No matching archived chats.") : tr("No archived chats on this page."));
            else
                state_->setText(total >= 0 ? tr("%1 matching chats · %2 on this page").arg(total).arg(items.size())
                                          : tr("%1 chats on this page").arg(items.size()));
            reload_->setText(tr("Refresh"));
            queryReady_ = true;
            syncActions();
        });
    }

    void unarchive()
    {
        const auto row = selection();
        mutation_ = true;
        syncActions();
        call("conversation.unarchive", {{"id", row.value("id")}, {"revision", row.value("revision")}},
             [this](const auto &, const auto &error) { finishMutation(error); });
    }

    void finishMutation(const QString &error)
    {
        mutation_ = false;
        if (error.isEmpty()) {
            changed_();
            resetQuery(false);
        } else {
            state_->setText(error);
            syncActions();
        }
    }

    void syncActions()
    {
        const bool idle = !loading_ && !mutation_ && !debounce_.isActive();
        const bool actionable = idle && queryReady_ && !selection().isEmpty();
        for (auto *filter : {repository_, date_, source_})
            filter->setEnabled(!mutation_);
        search_->setEnabled(!mutation_);
        unarchive_->setEnabled(actionable);
        erase_->setEnabled(actionable);
        first_->setEnabled(idle && hasCursor(before_));
        next_->setEnabled(idle && queryReady_ && hasCursor(nextBefore_));
        reload_->setEnabled(idle);
    }

    SettingsRequest request_;
    std::function<QJsonArray()> bindings_;
    std::function<void()> changed_;
    DeleteChat deleteChat_;
    QTimer debounce_;
    quint64 generation_ = 0;
    qint64 cutoff_ = 0;
    QJsonValue before_, nextBefore_;
    bool loading_ = false, mutation_ = false, queryReady_ = false;
    QLineEdit *search_ = nullptr;
    QLabel *state_ = nullptr;
    QComboBox *repository_ = nullptr, *date_ = nullptr, *source_ = nullptr;
    QListWidget *chats_ = nullptr;
    QPushButton *first_ = nullptr, *next_ = nullptr, *reload_ = nullptr, *unarchive_ = nullptr, *erase_ = nullptr;
};
}

QWidget *makeStorageSettings(RuntimeClient &client, SettingsRequest request, std::function<void()> openArchived)
{
    return new StoragePage(client, std::move(request), std::move(openArchived));
}

QWidget *makeArchivedChats(SettingsRequest request, std::function<QJsonArray()> bindings,
                           std::function<void()> conversationsChanged, DeleteChat deleteChat)
{
    return new ArchivedPage(std::move(request), std::move(bindings), std::move(conversationsChanged), std::move(deleteChat));
}
