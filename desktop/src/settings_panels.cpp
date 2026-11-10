#include "settings_panels.h"
#include "live_text.h"
#include "theme.h"
#include "main_window.h"

#include <QApplication>
#include <QClipboard>
#include <QComboBox>
#include <QDateTime>
#include <QDesktopServices>
#include <QDir>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QLocale>
#include <QMessageBox>
#include <QPainter>
#include <QPointer>
#include <QProgressBar>
#include <QPushButton>
#include <QResizeEvent>
#include <QScrollArea>
#include <QScrollBar>
#include <QSignalBlocker>
#include <QTimer>
#include <QToolButton>
#include <QUrl>
#include <QVBoxLayout>

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

QPushButton *button(live::Phrase text, const char *name, QBoxLayout &layout)
{
    auto *control = live::text(new QPushButton, std::move(text));
    control->setObjectName(QLatin1StringView(name));
    layout.addWidget(control);
    return control;
}

QVBoxLayout *storageCard(QVBoxLayout &parent)
{
    auto *frame = new QFrame;
    frame->setObjectName("settingsCard");
    auto *body = new QVBoxLayout(frame);
    body->setContentsMargins(20, 16, 20, 18);
    body->setSpacing(10);
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

QColor usageColor(const QPalette &palette, bool archived)
{
    return archived ? uiTone(UiTone::Info) : palette.color(QPalette::Highlight);
}

class StorageDot final : public QWidget {
public:
    explicit StorageDot(bool archived) : archived_(archived) { setFixedSize(9, 9); }

protected:
    void paintEvent(QPaintEvent *) override
    {
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing);
        painter.setPen(Qt::NoPen);
        painter.setBrush(usageColor(palette(), archived_));
        painter.drawEllipse(rect());
    }

private:
    bool archived_;
};

class StorageBar final : public QWidget {
public:
    StorageBar() { setFixedHeight(22); }
    void setParts(qint64 active, qint64 archived) { active_ = active; archived_ = archived; update(); }

protected:
    void paintEvent(QPaintEvent *) override
    {
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing);
        painter.setPen(Qt::NoPen);
        painter.setBrush(palette().color(QPalette::Midlight));
        const qreal total = qreal(active_) + qreal(archived_);
        if (total > 0)
            painter.setBrush(usageColor(palette(), active_ == 0));
        painter.drawRoundedRect(QRectF(rect()), height() / 2.0, height() / 2.0);
        if (active_ > 0 && archived_ > 0) {
            const qreal division = width() * qreal(active_) / total;
            painter.setClipRect(QRectF(division, 0, width() - division, height()));
            painter.setBrush(usageColor(palette(), true));
            painter.drawRoundedRect(QRectF(rect()), height() / 2.0, height() / 2.0);
        }
    }

private:
    qint64 active_ = 0, archived_ = 0;
};

class StoragePage final : public QWidget {
    Q_DECLARE_TR_FUNCTIONS(StoragePage)

public:
    StoragePage(RuntimeClient &client, SettingsRequest request)
        : request_(std::move(request))
    {
        setObjectName("storagePage");
        setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        auto *layout = new QVBoxLayout(this);
        layout->setContentsMargins(2, 8, 2, 8);
        layout->setSpacing(16);
        layout->addWidget(live::text(textLabel({}, "heading"), [] { return tr("Storage"); }));
        layout->addWidget(live::text(textLabel({}, "lead"), [] {
            return tr("See how much local message text is stored in your conversations.");
        }));
        auto *device = storageCard(*layout);
        auto *headline = new QHBoxLayout;
        headline->addWidget(live::text(textLabel({}, "sectionTitle"), [] { return tr("This device"); }), 1);
        refresh_ = button([] { return tr("Refresh"); }, "refreshChatStorage", *headline);
        refresh_->setProperty("actionRole", "secondary");
        refresh_->setIcon(uiIcon(UiIcon::Refresh));
        live::text(refresh_, [] { return tr("Refresh chat storage"); }, &QWidget::setAccessibleName);
        device->addLayout(headline);
        accounting_ = textLabel({}, "sectionTitle");
        accounting_->setObjectName("storageAccounting");
        accounting_->setTextInteractionFlags(Qt::TextSelectableByMouse);
        accounting_->hide();
        device->addWidget(accounting_);
        bar_ = new StorageBar;
        live::text(bar_, [] { return tr("Chat storage breakdown"); }, &QWidget::setAccessibleName);
        bar_->hide();
        device->addWidget(bar_);
        auto *legend = new QHBoxLayout;
        legend->setSpacing(20);
        const auto addLegend = [legend](live::Phrase heading, const char *name, bool archived) {
            auto *column = new QVBoxLayout;
            auto *labelRow = new QHBoxLayout;
            labelRow->setSpacing(8);
            labelRow->addWidget(new StorageDot(archived), 0, Qt::AlignVCenter);
            labelRow->addWidget(live::text(textLabel({}, "sectionTitle"), std::move(heading)), 1);
            column->addLayout(labelRow);
            auto *description = textLabel({});
            description->setObjectName(QLatin1StringView(name));
            column->addWidget(description);
            legend->addLayout(column, 1);
            return description;
        };
        activeUsage_ = addLegend([] { return tr("Active chats"); }, "storageActiveUsage", false);
        archivedUsage_ = addLegend([] { return tr("Archived chats"); }, "storageArchivedUsage", true);
        device->addLayout(legend);
        usageStatus_ = textLabel({});
        live::message(usageStatus_, [] { return tr("Waiting for the local runtime…"); });
        usageStatus_->setObjectName("storageUsageStatus");
        device->addWidget(usageStatus_);

        auto *toggle = button([] { return tr("Database maintenance"); }, "storageMaintenanceToggle", *layout);
        toggle->setIcon(uiIcon(UiIcon::Sliders));
        toggle->setCheckable(true);
        toggle->setProperty("actionRole", "secondary");
        auto *maintenanceBody = new QWidget;
        auto *groups = new QVBoxLayout(maintenanceBody);
        groups->setContentsMargins(0, 0, 0, 0);
        groups->setSpacing(12);
        auto *database = storageCard(*groups);
        database->parentWidget()->setObjectName("storageDatabaseActions");
        auto *databaseHeader = new QBoxLayout(QBoxLayout::LeftToRight);
        databaseHeader->addWidget(live::text(textLabel({}, "sectionTitle"), [] { return tr("Local database"); }), 1);
        inspect_ = button([] { return tr("Check database"); }, "inspectStorageDatabase", *databaseHeader);
        inspect_->setIcon(uiIcon(UiIcon::Storage));
        inspect_->setProperty("actionRole", "secondary");
        database->addLayout(databaseHeader);
        database->addWidget(live::text(textLabel({}), [] { return tr("Check the database to reveal its location and reusable space."); }));
        path_ = textLabel({});
        path_->setObjectName("storageDataRoot");
        path_->setTextInteractionFlags(Qt::TextSelectableByMouse);
        database->addWidget(path_);
        auto *pathRow = new QBoxLayout(QBoxLayout::LeftToRight);
        copy_ = button([] { return tr("Copy data path"); }, "copyStorageRoot", *pathRow);
        open_ = button([] { return tr("Open data folder"); }, "openStorageRoot", *pathRow);
        open_->setIcon(uiIcon(UiIcon::Folder));
        compact_ = button([] { return tr("Review compaction"); }, "compactStorage", *pathRow);
        for (auto *action : {copy_, open_, compact_})
            action->setProperty("actionRole", "secondary");
        database->addLayout(pathRow);

        auto *backupGroup = storageCard(*groups);
        backupGroup->parentWidget()->setObjectName("storageBackupActions");
        auto *backupHeader = new QBoxLayout(QBoxLayout::LeftToRight);
        backupHeader->addWidget(live::text(textLabel({}, "sectionTitle"), [] { return tr("Backups and verification"); }), 1);
        backup_ = button([] { return tr("Create backup"); }, "createStorageBackup", *backupHeader);
        backup_->setIcon(uiPrimaryIcon(UiIcon::Add));
        backup_->setProperty("primary", true);
        backupGroup->addLayout(backupHeader);
        backupGroup->addWidget(live::text(textLabel({}), [] { return tr("Verification does not restore or overwrite your data."); }));
        auto *backupRow = new QBoxLayout(QBoxLayout::LeftToRight);
        backups_ = new QComboBox;
        backups_->setObjectName("storageBackups");
        live::text(backups_, [] { return tr("Database or backup to verify"); }, &QWidget::setAccessibleName);
        backups_->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
        backups_->addItem(tr("Live database"));
        backupRow->addWidget(backups_, 1);
        verify_ = button([] { return tr("Verify selected"); }, "verifyStorageBackup", *backupRow);
        verify_->setIcon(uiIcon(UiIcon::Success));
        verify_->setProperty("actionRole", "secondary");
        backupGroup->addLayout(backupRow);

        layout->addWidget(maintenanceBody);
        maintenanceBody->hide();
        connect(toggle, &QPushButton::toggled, maintenanceBody, &QWidget::setVisible);
        auto *feedback = storageCard(*layout);
        feedback_ = feedback->parentWidget();
        feedback->addWidget(live::text(textLabel({}, "sectionTitle"), [] { return tr("Operation status"); }));
        status_ = textLabel({});
        status_->setObjectName("storageOperationStatus");
        feedback->addWidget(status_);
        progress_ = new QProgressBar;
        progress_->setObjectName("storageProgress");
        live::text(progress_, [] { return tr("Storage operation progress"); }, &QWidget::setAccessibleName);
        feedback->addWidget(progress_);
        auto *operationRow = new QHBoxLayout;
        operationRow->addStretch();
        cancel_ = button([] { return tr("Cancel operation"); }, "cancelStorageOperation", *operationRow);
        retry_ = button([] { return tr("Retry status"); }, "retryStorageOperation", *operationRow);
        cancel_->setProperty("actionRole", "secondary");
        retry_->setProperty("actionRole", "secondary");
        feedback->addLayout(operationRow);
        actionRows_ = {databaseHeader, pathRow, backupHeader, backupRow};
        layout->addStretch();
        connect(&live::language(), &live::Language::changed, this, [this] {
            backups_->setItemText(0, tr("Live database"));
            if (!lastUsage_.isEmpty())
                renderUsage(lastUsage_);
            arrangeActions();
        });

        poll_.setSingleShot(true);
        poll_.setInterval(250);
        connect(&poll_, &QTimer::timeout, this, [this] { poll(false); });
        connect(&client, &RuntimeClient::disconnected, this, [this] {
            ++generation_;
            ready_ = false;
            poll_.stop();
            if (!usageOperation_ && (pending_ || !operation_.isEmpty()))
                live::message(status_, [] { return tr("The runtime disconnected before the operation finished."); });
            live::message(usageStatus_, [] { return tr("Connect to the local runtime to refresh chat storage."); });
            operation_.clear();
            usageOperation_ = false;
            pending_ = false;
            statusFailed_ = false;
            cancellationQueued_ = false;
            cancellationInFlight_ = false;
            cancelling_ = false;
            freePages_ = 0;
            syncActions();
        });
        connect(refresh_, &QPushButton::clicked, this, [this] { start("storage.chats"); });
        connect(inspect_, &QPushButton::clicked, this, [this] { start("storage.scan"); });
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
                live::message(status_, [] { return tr("Could not open the data folder."); });
            syncActions();
        });
        syncActions();
    }

    void runtimeReady()
    {
        ready_ = true;
        start("storage.chats");
    }

protected:
    void resizeEvent(QResizeEvent *event) override
    {
        QWidget::resizeEvent(event);
        arrangeActions();
    }

    void showEvent(QShowEvent *event) override
    {
        QWidget::showEvent(event);
        if (!operation_.isEmpty() && !pending_ && !statusFailed_)
            poll_.start();
        else if (operation_.isEmpty())
            start("storage.chats");
    }
    void hideEvent(QHideEvent *event) override
    {
        if (!usageOperation_)
            poll_.stop();
        QWidget::hideEvent(event);
    }

private:
    void arrangeActions()
    {
        for (auto *row : actionRows_) {
            int required = qMax(0, row->count() - 1) * row->spacing();
            for (int index = 0; index < row->count(); ++index)
                required += row->itemAt(index)->sizeHint().width();
            row->setDirection(required + 52 > width() ? QBoxLayout::TopToBottom : QBoxLayout::LeftToRight);
        }
    }

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
        if (!ready_ || pending_ || !operation_.isEmpty())
            return;
        usageOperation_ = method == "storage.chats";
        if (usageOperation_) {
            live::message(usageStatus_, [] { return tr("Measuring chat storage…"); });
        } else {
            live::message(status_, [] { return tr("Submitting storage operation…"); });
            progress_->setRange(0, 0);
        }
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
            if (usageOperation_) {
                live::message(usageStatus_, [error] {
                    return error.isEmpty() ? tr("Chat storage measurement is unavailable.") : error;
                });
                operation_.clear();
                usageOperation_ = false;
                syncActions();
                return;
            }
            cancellationQueued_ |= std::exchange(cancellationInFlight_, false);
            statusFailed_ = true;
            live::message(status_, [error] {
                return error.isEmpty() ? tr("Storage operation status is unavailable.") : error;
            });
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
            if (usageOperation_) {
                renderUsage(result);
                live::message(usageStatus_, [this] {
                    const auto count = lastUsage_.value("active").toObject().value("conversations").toInteger()
                        + lastUsage_.value("archived").toObject().value("conversations").toInteger();
                    return count > 0 ? tr("Updated %1").arg(dateText(lastUsage_.value("measured_at").toInteger()))
                                     : tr("No local conversations yet.");
                });
            } else {
                renderMaintenance(kind, result);
            }
            operation_.clear();
        } else if (state == "FAILED" || state == "CANCELLED") {
            const auto issue = reply.value("error").toObject();
            live::message(usageOperation_ ? usageStatus_ : status_,
                       [state, issue] {
                return state == "CANCELLED" ? tr("Storage operation cancelled.")
                    : tr("Storage operation failed: %1").arg(issue.value("message").toString());
            });
            operation_.clear();
        } else if (state == "CANCEL_REQUESTED") {
            cancellationQueued_ = false;
            live::message(status_, [] { return tr("Waiting for the storage worker to stop…"); });
        } else {
            if (!usageOperation_)
                live::message(status_, [phase, state] {
                    return tr("Storage operation: %1").arg(uiStatusLabel(phase.isEmpty() ? state : phase));
                });
            if (cancellationQueued_) {
                poll(true);
                return;
            }
        }
        if (operation_.isEmpty()) {
            cancellationQueued_ = false;
            usageOperation_ = false;
        } else if (isVisible() || usageOperation_) {
            poll_.start();
        }
        syncActions();
    }

    void renderUsage(const QJsonObject &usage)
    {
        lastUsage_ = usage;
        const auto active = usage.value("active").toObject();
        const auto archived = usage.value("archived").toObject();
        const auto activeBytes = active.value("message_bytes").toInteger();
        const auto archivedBytes = archived.value("message_bytes").toInteger();
        const auto activeCount = active.value("conversations").toInteger();
        const auto archivedCount = archived.value("conversations").toInteger();
        const auto caption = [](qint64 bytes, qint64 count) {
            const auto conversations = count == 1 ? tr("1 conversation")
                                                   : tr("%1 conversations").arg(QLocale().toString(count));
            return QStringLiteral("%1 · %2").arg(sizeText(bytes), conversations);
        };
        accounting_->setText(tr("%1 of local message text").arg(sizeText(activeBytes + archivedBytes)));
        accounting_->show();
        bar_->setParts(activeBytes, archivedBytes);
        bar_->show();
        activeUsage_->setText(caption(activeBytes, activeCount));
        archivedUsage_->setText(caption(archivedBytes, archivedCount));
    }

    void renderScan(const QJsonObject &snapshot)
    {
        const auto logical = snapshot.value("logical").toObject();
        pageSize_ = logical.value("page_size").toInteger();
        pageCount_ = logical.value("page_count").toInteger();
        freePages_ = logical.value("free_pages").toInteger();
        path_->setText(snapshot.value("data_root").toString());
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

    void renderMaintenance(const QString &kind, const QJsonObject &result)
    {
        if (kind == "SCAN")
            renderScan(result);
        const auto backup = result.value("backup").toString();
        if (!backup.isEmpty()) {
            if (backups_->findData(backup) < 0)
                backups_->addItem(backup, backup);
            backups_->setCurrentIndex(backups_->findData(backup));
            live::message(status_, [backup] { return tr("Storage operation completed. Backup: %1").arg(backup); });
        } else if (kind == "COMPACT") {
            freePages_ = 0;
            live::message(status_, [result] {
                return tr("Storage operation completed. Database pages: %1 before, %2 after.")
                    .arg(result.value("before_page_count").toInteger()).arg(result.value("after_page_count").toInteger());
            });
        } else {
            live::message(status_, [] { return tr("Storage operation completed."); });
        }
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
        const bool idle = ready_ && !pending_ && operation_.isEmpty();
        for (auto *control : {refresh_, inspect_, backup_, verify_})
            control->setEnabled(idle);
        compact_->setEnabled(idle && freePages_ > 0);
        const auto hasPath = QDir::isAbsolutePath(path_->text());
        path_->setVisible(hasPath);
        copy_->setEnabled(hasPath);
        open_->setEnabled(hasPath);
        const bool maintenanceBusy = !usageOperation_ && (!operation_.isEmpty() || pending_);
        progress_->setVisible(maintenanceBusy && !statusFailed_);
        cancel_->setVisible(maintenanceBusy && !statusFailed_);
        cancel_->setEnabled(!operation_.isEmpty() && !cancellationQueued_ &&
                            !cancellationInFlight_ && !cancelling_ && !statusFailed_);
        retry_->setVisible(!operation_.isEmpty() && statusFailed_);
        feedback_->setVisible(maintenanceBusy || !status_->text().isEmpty());
    }

    SettingsRequest request_;
    QJsonObject lastUsage_;
    QList<QBoxLayout *> actionRows_;
    QTimer poll_;
    quint64 generation_ = 0;
    QString operation_;
    bool ready_ = false, usageOperation_ = false;
    bool pending_ = false;
    bool cancellationQueued_ = false;
    bool cancellationInFlight_ = false;
    bool cancelling_ = false;
    bool statusFailed_ = false;
    qint64 pageSize_ = 0, pageCount_ = 0, freePages_ = 0;
    QLabel *accounting_ = nullptr, *activeUsage_ = nullptr, *archivedUsage_ = nullptr, *usageStatus_ = nullptr;
    QLabel *path_ = nullptr, *status_ = nullptr;
    QWidget *feedback_ = nullptr;
    StorageBar *bar_ = nullptr;
    QProgressBar *progress_ = nullptr;
    QComboBox *backups_ = nullptr;
    QPushButton *refresh_ = nullptr, *inspect_ = nullptr, *copy_ = nullptr, *open_ = nullptr, *backup_ = nullptr,
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
        layout->setContentsMargins(0, 0, 0, 0);
        scroll_ = new QScrollArea;
        scroll_->setWidgetResizable(true);
        scroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
        live::text(scroll_, [] { return tr("Archived chats"); }, &QWidget::setAccessibleName);
        layout->addWidget(scroll_);
        auto *host = new QWidget;
        auto *centering = new QHBoxLayout(host);
        centering->setContentsMargins(0, 0, 0, 0);
        auto *body = new QWidget;
        body->setMaximumWidth(960);
        centering->addWidget(body, 1);
        auto *bodyLayout = new QVBoxLayout(body);
        bodyLayout->setContentsMargins(6, 8, 14, 12);
        bodyLayout->setSpacing(16);
        scroll_->setWidget(host);
        bodyLayout->addWidget(live::text(textLabel({}, "heading"), [] { return tr("Archived chats"); }));
        bodyLayout->addWidget(live::text(textLabel({}, "lead"), [] { return tr("Restore a chat to the sidebar or review it for permanent deletion."); }));

        search_ = new QLineEdit;
        search_->setObjectName("archivedChatSearch");
        search_->setProperty("searchField", true);
        search_->setClearButtonEnabled(true);
        search_->setMaxLength(120);
        live::placeholder(search_, [] { return tr("Search archived chats"); });
        live::text(search_, [] { return tr("Search archived chats"); }, &QWidget::setAccessibleName);
        search_->setMinimumWidth(0);
        filters_ = new QGridLayout;
        filters_->setSpacing(8);
        bodyLayout->addLayout(filters_);
        repository_ = new QComboBox;
        repository_->setObjectName("archivedRepositoryFilter");
        live::text(repository_, [] { return tr("Repository filter"); }, &QWidget::setAccessibleName);
        repository_->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
        repository_->setMinimumContentsLength(10);
        date_ = new QComboBox;
        date_->setObjectName("archivedDateFilter");
        live::text(date_, [] { return tr("Archive date filter"); }, &QWidget::setAccessibleName);
        live::item(date_, [] { return tr("Any date"); }, 0);
        live::item(date_, [] { return tr("Older than 30 days"); }, 30);
        live::item(date_, [] { return tr("Older than 90 days"); }, 90);
        source_ = new QComboBox;
        source_->setObjectName("archivedSourceFilter");
        live::text(source_, [] { return tr("Chat source filter"); }, &QWidget::setAccessibleName);
        live::item(source_, [] { return tr("Any source"); });
        live::item(source_, [] { return tr("Desktop"); }, "DASHBOARD");
        live::item(source_, [] { return tr("Automation"); }, "AUTOMATION");
        arrangeFilters();

        auto *scopeRow = new QHBoxLayout;
        scope_ = textLabel(tr("All repositories"), "sectionTitle");
        scopeRow->addWidget(scope_, 1);
        reload_ = button([] { return tr("Refresh"); }, "archivedRefresh", *scopeRow);
        reload_->setProperty("actionRole", "secondary");
        reload_->setIcon(uiIcon(UiIcon::Refresh));
        bodyLayout->addLayout(scopeRow);
        state_ = textLabel({});
        state_->setObjectName("archivedChatsState");
        bodyLayout->addWidget(state_);
        cards_ = new QWidget;
        cards_->setObjectName("archivedChats");
        cardsLayout_ = new QVBoxLayout(cards_);
        cardsLayout_->setContentsMargins(0, 0, 0, 0);
        cardsLayout_->setSpacing(8);
        bodyLayout->addWidget(cards_);
        auto *paging = new QHBoxLayout;
        first_ = button([] { return tr("First page"); }, "archivedFirstPage", *paging);
        first_->setProperty("actionRole", "secondary");
        next_ = button([] { return tr("Next page"); }, "archivedNextPage", *paging);
        next_->setProperty("actionRole", "secondary");
        paging->addStretch();
        bodyLayout->addLayout(paging);
        bodyLayout->addStretch();
        connect(&live::language(), &live::Language::changed, this, [this] {
            if (repository_->count())
                repository_->setItemText(0, tr("All repositories"));
            scope_->setText(repository_->count() ? repository_->currentText() : tr("All repositories"));
            reload_->setText(reload_->property("retry").toBool() ? tr("Retry") : tr("Refresh"));
            arrangeFilters();
        });

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
        syncActions();
    }

protected:
    void resizeEvent(QResizeEvent *event) override
    {
        QWidget::resizeEvent(event);
        arrangeFilters();
    }

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
        ++generation_;
        queryReady_ = false;
        QWidget::hideEvent(event);
    }

private:
    static bool hasCursor(const QJsonValue &value) { return !value.isNull() && !value.isUndefined(); }

    void arrangeFilters()
    {
        const auto available = width();
        const int columns = available >= 830 ? 4 : available >= 520 ? 3 : available >= 345 ? 2 : 1;
        if (columns == filterColumns_)
            return;
        filterColumns_ = columns;
        for (auto *widget : std::array<QWidget *, 4>{search_, repository_, source_, date_})
            filters_->removeWidget(widget);
        filters_->setColumnStretch(0, 1);
        if (columns == 4) {
            filters_->addWidget(search_, 0, 0);
            filters_->addWidget(repository_, 0, 1);
            filters_->addWidget(source_, 0, 2);
            filters_->addWidget(date_, 0, 3);
        } else if (columns == 3) {
            filters_->addWidget(search_, 0, 0, 1, 3);
            filters_->addWidget(repository_, 1, 0);
            filters_->addWidget(source_, 1, 1);
            filters_->addWidget(date_, 1, 2);
        } else if (columns == 2) {
            filters_->addWidget(search_, 0, 0, 1, 2);
            filters_->addWidget(repository_, 1, 0, 1, 2);
            filters_->addWidget(source_, 2, 0);
            filters_->addWidget(date_, 2, 1);
        } else {
            filters_->addWidget(search_, 0, 0);
            filters_->addWidget(repository_, 1, 0);
            filters_->addWidget(source_, 2, 0);
            filters_->addWidget(date_, 3, 0);
        }
    }

    void clearCards()
    {
        while (auto *item = cardsLayout_->takeAt(0)) {
            if (auto *row = item->widget()) {
                row->hide();
                row->setObjectName({});
                row->deleteLater();
            }
            delete item;
        }
    }

    void addCard(const QJsonObject &chat)
    {
        const auto id = chat.value("id").toString();
        const auto title = chat.value("title").toString();
        if (id.isEmpty())
            return;
        auto *card = new QFrame;
        card->setObjectName("settingsCard");
        card->setProperty("conversationId", id);
        auto *row = new QHBoxLayout(card);
        row->setContentsMargins(16, 12, 12, 12);
        row->setSpacing(8);
        auto *description = new QVBoxLayout;
        description->setSpacing(4);
        description->addWidget(textLabel(title, "sectionTitle"));
        const auto timestamp = chat.value("updated_at").toInteger();
        if (timestamp > 0)
            description->addWidget(live::text(textLabel({}), [timestamp] { return dateText(timestamp); }));
        row->addLayout(description, 1);
        auto *erase = new QToolButton;
        erase->setObjectName("deleteArchivedChat");
        erase->setProperty("actionRole", "danger");
        erase->setIcon(uiIcon(UiIcon::Trash));
        erase->setIconSize({18, 18});
        live::text(erase, [] { return tr("Review deletion"); }, &QWidget::setToolTip);
        live::text(erase, [title] { return tr("Delete %1").arg(title); }, &QWidget::setAccessibleName);
        erase->setFocusPolicy(Qt::StrongFocus);
        row->addWidget(erase);
        auto *restore = button([] { return tr("Unarchive"); }, "unarchiveChat", *row);
        restore->setProperty("actionRole", "secondary");
        live::text(restore, [title] { return tr("Unarchive %1").arg(title); }, &QWidget::setAccessibleName);
        cardsLayout_->addWidget(card);
        connect(erase, &QToolButton::clicked, this, [this, id] { eraseChat(id); });
        connect(restore, &QPushButton::clicked, this, [this, chat] { unarchive(chat); });
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
            live::message(state_, [] { return tr("Loading archived chats…"); });
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
        scope_->setText(repository_->currentText());
        const auto query = search_->text().trimmed();
        if (query.toUtf8().size() > 160) {
            clearCards();
            live::message(state_, [] { return tr("Shorten the search to 160 UTF-8 bytes."); });
            syncActions();
            return;
        }
        loading_ = true;
        queryReady_ = false;
        live::message(state_, [] { return tr("Loading archived chats…"); });
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
                clearCards();
                live::message(state_, [error] {
                    return error.isEmpty() ? tr("Could not load archived chats.")
                                           : tr("Could not load archived chats: %1").arg(error);
                });
                reload_->setProperty("retry", true);
                reload_->setText(tr("Retry"));
                syncActions();
                return;
            }
            const auto items = result.value("items").toArray();
            nextBefore_ = result.value("next_before");
            clearCards();
            for (const auto &value : items)
                addCard(value.toObject());
            scroll_->verticalScrollBar()->setValue(0);
            const auto total = result.value("total").toInteger(-1);
            live::message(state_, [total, count = items.size()] {
                if (!count)
                    return total == 0 ? tr("No matching archived chats.") : tr("No archived chats on this page.");
                return total >= 0 ? tr("%1 matching chats · %2 on this page").arg(total).arg(count)
                                  : tr("%1 chats on this page").arg(count);
            });
            reload_->setProperty("retry", false);
            reload_->setText(tr("Refresh"));
            queryReady_ = true;
            syncActions();
        });
    }

    void unarchive(const QJsonObject &row)
    {
        if (mutation_ || !queryReady_)
            return;
        mutation_ = true;
        syncActions();
        call("conversation.unarchive", {{"id", row.value("id")}, {"revision", row.value("revision")}},
             [this](const auto &, const auto &error) { finishMutation(error); });
    }

    void eraseChat(const QString &id)
    {
        if (mutation_ || !queryReady_)
            return;
        mutation_ = true;
        syncActions();
        deleteChat_(id, [owner = QPointer(this)](bool success) {
            if (!owner)
                return;
            owner->mutation_ = false;
            if (success) {
                if (owner->isVisible())
                    owner->search_->setFocus(Qt::OtherFocusReason);
                owner->resetQuery(false);
            } else {
                owner->syncActions();
            }
        });
    }

    void finishMutation(const QString &error)
    {
        mutation_ = false;
        if (error.isEmpty()) {
            changed_();
            if (isVisible())
                search_->setFocus(Qt::OtherFocusReason);
            resetQuery(false);
        } else {
            live::message(state_, [error] { return error; });
            syncActions();
        }
    }

    void syncActions()
    {
        const bool idle = !loading_ && !mutation_ && !debounce_.isActive();
        for (auto *filter : {repository_, date_, source_})
            filter->setEnabled(!mutation_);
        search_->setEnabled(!mutation_);
        cards_->setEnabled(idle && queryReady_);
        first_->setVisible(hasCursor(before_));
        next_->setVisible(queryReady_ && hasCursor(nextBefore_));
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
    int filterColumns_ = 0;
    QJsonValue before_, nextBefore_;
    bool loading_ = false, mutation_ = false, queryReady_ = false;
    QLineEdit *search_ = nullptr;
    QLabel *state_ = nullptr, *scope_ = nullptr;
    QComboBox *repository_ = nullptr, *date_ = nullptr, *source_ = nullptr;
    QScrollArea *scroll_ = nullptr;
    QWidget *cards_ = nullptr;
    QVBoxLayout *cardsLayout_ = nullptr;
    QGridLayout *filters_ = nullptr;
    QPushButton *first_ = nullptr, *next_ = nullptr, *reload_ = nullptr;
};
}

QWidget *makeStorageSettings(RuntimeClient &client, MainWindow &window, SettingsRequest request)
{
    auto *page = new StoragePage(client, std::move(request));
    QObject::connect(&window, &MainWindow::localRuntimeReady, page, [page] { page->runtimeReady(); });
    auto *host = new QWidget;
    auto *layout = new QHBoxLayout(host);
    layout->setContentsMargins(0, 0, 0, 0);
    page->setMaximumWidth(960);
    layout->addStretch();
    layout->addWidget(page, 100, Qt::AlignTop);
    layout->addStretch();
    return host;
}

QWidget *makeArchivedChats(SettingsRequest request, std::function<QJsonArray()> bindings,
                           std::function<void()> conversationsChanged, DeleteChat deleteChat)
{
    return new ArchivedPage(std::move(request), std::move(bindings), std::move(conversationsChanged), std::move(deleteChat));
}
