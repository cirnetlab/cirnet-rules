#include "main_window.h"
#include "compact_text_edit.h"
#include "repository_picker.h"

#include <QAction>
#include <QComboBox>
#include <QDateTime>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QItemSelectionModel>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QLocale>
#include <QMenu>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPointer>
#include <QPushButton>
#include <QScreen>
#include <QScrollBar>
#include <QShortcut>
#include <QSignalBlocker>
#include <QStackedWidget>
#include <QTextCursor>
#include <QTimer>
#include <QUuid>
#include <QVariant>
#include <QVBoxLayout>
#include <utility>

namespace {
constexpr int IdRole = Qt::UserRole;
constexpr int RecordRole = Qt::UserRole + 1;
constexpr int conversationPageSize = 30;
constexpr int conversationCacheLimit = 300;
constexpr int draftLimit = 128;
constexpr int draftBytes = 8192;
constexpr int transcriptLimit = 48;
constexpr int transcriptBlockLimit = 3500;

QString requestKey()
{
    return QUuid::createUuid().toString(QUuid::WithoutBraces);
}

QLabel *label(const QString &text, const char *name)
{
    auto *item = new QLabel(text);
    item->setObjectName(QString::fromLatin1(name));
    item->setTextFormat(Qt::PlainText);
    item->setWordWrap(true);
    return item;
}

bool hasCursor(const QJsonValue &cursor)
{
    return !cursor.isUndefined() && !cursor.isNull();
}

QString displayedMessage(const QJsonObject &row, const QString &you, const QString &shortened)
{
    auto content = row.value("content").toString();
    int end = -1;
    for (int line = 0; line < 400; ++line) {
        end = content.indexOf(u'\n', end + 1);
        if (end < 0)
            break;
    }
    if (end >= 0)
        content = content.left(end) + QStringLiteral("\n") + shortened;
    const auto role = row.value("role").toString();
    return (role == "USER" ? you : role) + QStringLiteral("\n") + content;
}

void reactivate(QDialog *dialog)
{
    dialog->show();
    dialog->raise();
    dialog->activateWindow();
}
}

RepositoryPicker::RepositoryPicker(bool multiple, const QString &selected, QWidget *parent)
    : QWidget(parent), search_(new QLineEdit), list_(new QListWidget),
      manage_(new QPushButton(tr("Manage repositories"))),
      status_(label(tr("Loading repository catalog…"), "meta")), preselected_(selected)
{
    auto *layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(6);
    search_->setObjectName("repositoryPickerSearch");
    search_->setAccessibleName(tr("Search repository bindings"));
    search_->setPlaceholderText(tr("Search by repository, path or branch"));
    search_->setClearButtonEnabled(true);
    manage_->setObjectName("manageRepositoryPicker");
    auto *tools = new QHBoxLayout;
    tools->addWidget(search_, 1);
    tools->addWidget(manage_);
    layout->addLayout(tools);
    list_->setObjectName("repositoryPickerList");
    list_->setSelectionMode(multiple ? QAbstractItemView::MultiSelection : QAbstractItemView::SingleSelection);
    list_->setAccessibleName(tr("Repository bindings"));
    list_->setMinimumHeight(96);
    list_->setTextElideMode(Qt::ElideMiddle);
    layout->addWidget(list_, 1);
    layout->addWidget(status_);
    connect(search_, &QLineEdit::textChanged, this, &RepositoryPicker::filter);
    connect(list_, &QListWidget::itemSelectionChanged, this, &RepositoryPicker::filter);
}

QJsonArray RepositoryPicker::selection() const
{
    QJsonArray ids;
    if (loaded_)
        for (const auto *row : list_->selectedItems())
            ids.append(row->data(IdRole).toString());
    return ids;
}

void RepositoryPicker::populate(const QJsonArray &catalog, bool loaded)
{
    QJsonArray ids;
    for (const auto *row : list_->selectedItems())
        ids.append(row->data(IdRole).toString());
    if (ids.isEmpty() && !preselected_.isEmpty())
        ids.append(preselected_);
    loaded_ = loaded;
    if (!loaded_) {
        list_->setEnabled(false);
        status_->setText(tr("The repository catalog could not be loaded. Reconnect or open Manage repositories."));
        return;
    }
    preselected_.clear();
    list_->setEnabled(true);
    const QSignalBlocker blocker(list_);
    list_->clear();
    for (const auto &value : catalog) {
        const auto item = value.toObject();
        const auto id = item.value("binding_id").toString();
        const auto state = item.value("status").toString();
        if (item.value("archived").toBool() || state == "ARCHIVED" || id.isEmpty())
            continue;
        const auto branch = item.value("branch").toString(tr("Detached"));
        auto *row = new QListWidgetItem(tr("%1 · %2").arg(item.value("display_name").toString(), branch), list_);
        row->setToolTip((item.value("root").toString() + QStringLiteral("\n") + state).toHtmlEscaped());
        row->setData(IdRole, id);
        row->setData(RecordRole, item);
        if (state == "MISSING" || state == "INACCESSIBLE" || state == "REBIND_REQUIRED") {
            row->setFlags(row->flags() & ~Qt::ItemIsEnabled);
            row->setText(tr("%1 · %2").arg(row->text(), state));
        } else {
            row->setSelected(ids.contains(id));
        }
    }
    filter();
}

void RepositoryPicker::filter()
{
    if (!loaded_)
        return;
    const auto query = search_->text().trimmed();
    int shown = 0;
    for (int index = 0; index < list_->count(); ++index) {
        auto *row = list_->item(index);
        const auto item = row->data(RecordRole).toJsonObject();
        const auto match = row->text().contains(query, Qt::CaseInsensitive)
            || item.value("root").toString().contains(query, Qt::CaseInsensitive)
            || item.value("file_hint").toString().contains(query, Qt::CaseInsensitive);
        row->setHidden(!match);
        shown += match ? 1 : 0;
    }
    status_->setText(list_->count() == 0 ? tr("Register a local Git repository to continue.")
        : shown == 0 ? tr("No repository matches this search.")
        : tr("%1 shown · %2 selected").arg(shown).arg(list_->selectedItems().size()));
}

QWidget *MainWindow::makeConversationHistory()
{
    auto *history = new QWidget;
    history->setObjectName("conversationHistory");
    auto *layout = new QVBoxLayout(history);
    layout->setContentsMargins(0, 8, 0, 0);
    layout->setSpacing(6);
    layout->addWidget(label(tr("Conversations"), "sectionTitle"));
    conversationSearch_ = new QLineEdit;
    conversationSearch_->setObjectName("conversationSearch");
    conversationSearch_->setPlaceholderText(tr("Search conversations"));
    conversationSearch_->setAccessibleName(tr("Search conversations"));
    conversationSearch_->setClearButtonEnabled(true);
    conversationList_ = new QListWidget;
    conversationList_->setObjectName("conversationList");
    conversationList_->setAccessibleName(tr("Conversation history"));
    conversationList_->setWordWrap(false);
    conversationList_->setUniformItemSizes(true);
    conversationList_->setSelectionMode(QAbstractItemView::NoSelection);
    conversationList_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    conversationList_->setTextElideMode(Qt::ElideRight);
    conversationList_->setContextMenuPolicy(Qt::CustomContextMenu);
    auto *state = label(tr("Connect to load saved chats."), "meta");
    state->setObjectName("conversationHistoryState");
    layout->addWidget(conversationSearch_);
    layout->addWidget(state);
    layout->addWidget(conversationList_, 1);

    auto *searchDelay = new QTimer(history);
    searchDelay->setSingleShot(true);
    searchDelay->setInterval(220);
    connect(conversationSearch_, &QLineEdit::textChanged, this, [this, searchDelay] {
        ++conversationRequest_;
        conversationLoading_ = true;
        searchDelay->start();
    });
    connect(searchDelay, &QTimer::timeout, this, [this] { refreshConversations(); });
    auto *scroll = conversationList_->verticalScrollBar();
    connect(scroll, &QScrollBar::valueChanged, this, [this, scroll](int value) {
        if (conversationList_->isVisible() && scroll->maximum() > 0 && value == scroll->maximum())
            refreshConversations(true);
    });
    connect(conversationList_, &QListWidget::itemActivated, this, [this](QListWidgetItem *item) {
        navigateConversation(item->data(IdRole).toString());
    });
    connect(conversationList_, &QListWidget::itemClicked, this, [this](QListWidgetItem *item) {
        navigateConversation(item->data(IdRole).toString());
    });
    connect(conversationList_, &QListWidget::customContextMenuRequested, this, [this](const QPoint &pos) {
        const auto *row = conversationList_->itemAt(pos);
        if (!row)
            row = conversationList_->currentItem();
        QMenu menu(this);
        const auto record = row ? row->data(RecordRole).toJsonObject() : QJsonObject{};
        QAction *resume = nullptr;
        QAction *rename = nullptr;
        QAction *archive = nullptr;
        QAction *remove = nullptr;
        if (row) {
            const bool active = online_ && record.value("state").toString() != "ARCHIVED";
            resume = menu.addAction(tr("Open conversation"));
            rename = menu.addAction(tr("Rename"));
            archive = menu.addAction(tr("Archive"));
            remove = menu.addAction(tr("Delete conversation…"));
            resume->setEnabled(online_);
            rename->setEnabled(active);
            archive->setEnabled(active);
            remove->setEnabled(online_);
        }
        QAction *more = nullptr;
        QAction *newest = nullptr;
        if (conversationList_->property("historyWindowMoved").toBool())
            newest = menu.addAction(tr("Show newest conversations"));
        if (canLoadMoreConversations()) {
            if (row)
                menu.addSeparator();
            more = menu.addAction(tr("Load more"));
        }
        const auto choice = menu.isEmpty() ? nullptr : menu.exec(conversationList_->viewport()->mapToGlobal(pos));
        if (!choice)
            return;
        if (choice == more) {
            refreshConversations(true);
        } else if (choice == newest) {
            conversationList_->setProperty("historyWindowMoved", false);
            conversationList_->setProperty("historyPaged", false);
            refreshConversations();
        } else if (choice == resume) {
            navigateConversation(record.value("id").toString());
        } else if (choice == rename || choice == archive) {
            editConversation(choice == archive, record);
        } else if (choice == remove) {
            deleteConversationData(record.value("id").toString());
        }
    });
    return history;
}

QWidget *MainWindow::makeConversations()
{
    auto *page = new QWidget;
    page->setObjectName("conversationsPage");
    auto *outer = new QHBoxLayout(page);
    outer->setContentsMargins(24, 24, 24, 20);
    outer->addStretch();
    auto *panel = new QWidget;
    panel->setMaximumWidth(950);
    panel->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    outer->addWidget(panel, 100);
    outer->addStretch();
    auto *layout = new QVBoxLayout(panel);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(10);

    auto *header = new QHBoxLayout;
    conversationTitle_ = label(tr("Chat"), "heading");
    header->addWidget(conversationTitle_, 1);
    conversationActions_ = new QPushButton(tr("Actions"));
    conversationActions_->setObjectName("conversationActions");
    auto *menu = new QMenu(conversationActions_);
    auto *rename = menu->addAction(tr("Rename"));
    auto *archive = menu->addAction(tr("Archive"));
    auto *remove = menu->addAction(tr("Delete conversation…"));
    remove->setObjectName("deleteConversation");
    connect(rename, &QAction::triggered, this, [this] { editConversation(false); });
    connect(archive, &QAction::triggered, this, [this] { editConversation(true); });
    connect(remove, &QAction::triggered, this, [this] { deleteConversationData(currentConversation_); });
    conversationActions_->setMenu(menu);
    header->addWidget(conversationActions_);
    layout->addLayout(header);
    conversationDetail_ = label(tr("Select a saved conversation or start a new chat."), "conversationDetail");
    layout->addWidget(conversationDetail_);

    auto *actions = new QHBoxLayout;
    auto *start = new QPushButton(tr("New Chat"));
    start->setObjectName("emptyNewChat");
    start->setProperty("primary", true);
    connect(start, &QPushButton::clicked, this, [this] {
        if (newChatAction_)
            newChatAction_->trigger();
    });
    actions->addWidget(start);
    auto *taskDetails = new QPushButton(tr("Task details"));
    taskDetails->setObjectName("conversationTaskDetails");
    cancelTask_ = new QPushButton(tr("Cancel pending task"));
    actions->addWidget(taskDetails);
    actions->addWidget(cancelTask_);
    olderMessages_ = new QPushButton(tr("Load older messages"));
    olderMessages_->setObjectName("loadOlderMessages");
    actions->addWidget(olderMessages_);
    connect(olderMessages_, &QPushButton::clicked, this, [this] {
        openConversation(currentConversation_, TranscriptPage::Older);
    });
    latestMessages_ = new QPushButton(tr("Latest messages"));
    latestMessages_->setObjectName("loadLatestMessages");
    actions->addWidget(latestMessages_);
    connect(latestMessages_, &QPushButton::clicked, this, [this] {
        openConversation(currentConversation_);
    });
    actions->addStretch();
    layout->addLayout(actions);

    transcript_ = new QPlainTextEdit;
    transcript_->setObjectName("conversationTranscript");
    transcript_->setReadOnly(true);
    transcript_->setAccessibleName(tr("Conversation transcript"));
    transcript_->setPlaceholderText(tr("Saved messages and task outcomes appear here."));
    transcript_->setContextMenuPolicy(Qt::CustomContextMenu);
    connect(transcript_, &QWidget::customContextMenuRequested, this, [this](const QPoint &position) {
        auto *menu = transcript_->createStandardContextMenu(position);
        menu->setAttribute(Qt::WA_DeleteOnClose);
        const int positionInText = transcript_->cursorForPosition(position).position();
        int start = 0;
        QString messageId;
        for (const auto &row : transcriptRows_) {
            const auto length = displayedMessage(row, tr("You"), tr("Display shortened.")).size();
            if (positionInText >= start && positionInText < start + length) {
                messageId = row.value("id").toString();
                break;
            }
            start += int(length) + 2;
        }
        if (online_ && !messageId.isEmpty()) {
            const auto id = currentConversation_;
            menu->addSeparator();
            auto *remove = menu->addAction(tr("Delete submitted message…"));
            remove->setObjectName("deleteSubmittedMessage");
            connect(remove, &QAction::triggered, this, [this, id, messageId] {
                deleteConversationData(id, messageId);
            });
        }
        menu->popup(transcript_->viewport()->mapToGlobal(position));
    });
    layout->addWidget(transcript_, 1);
    composer_ = new CompactTextEdit(5);
    composer_->setObjectName("conversationComposer");
    composer_->setAccessibleName(tr("Message"));
    composer_->setPlaceholderText(tr("Describe a scoped task…"));
    layout->addWidget(composer_);
    auto *commands = new QHBoxLayout;
    commands->addWidget(label(tr("Local handoff · Ctrl+Enter to send"), "meta"), 1);
    auto *clear = new QPushButton(tr("Clear draft"));
    clear->setObjectName("clearDraft");
    auto *restore = new QPushButton(tr("Undo clear"));
    restore->setObjectName("restoreDraft");
    commands->addWidget(clear);
    commands->addWidget(restore);
    connect(clear, &QPushButton::clicked, this, &MainWindow::clearCurrentDraft);
    connect(restore, &QPushButton::clicked, this, &MainWindow::restoreCurrentDraft);
    send_ = new QPushButton(tr("Send"));
    send_->setObjectName("sendMessage");
    send_->setProperty("primary", true);
    commands->addWidget(send_);
    layout->addLayout(commands);

    auto *shortcut = new QShortcut(QKeySequence(Qt::CTRL | Qt::Key_Return), composer_);
    shortcut->setContext(Qt::WidgetWithChildrenShortcut);
    connect(shortcut, &QShortcut::activated, this, &MainWindow::sendMessage);
    connect(composer_, &QPlainTextEdit::textChanged, this, [this] {
        if (!currentConversation_.isEmpty()) {
            const auto draft = composer_->toPlainText();
            if (draft.toUtf8().size() > draftBytes) {
                const QSignalBlocker guard(composer_);
                composer_->setPlainText(conversationDrafts_.value(currentConversation_));
                composer_->moveCursor(QTextCursor::End);
                showError(tr("Drafts are limited to 8 KiB."));
            } else if (draft.isEmpty()) {
                conversationDrafts_.remove(currentConversation_);
            } else {
                conversationDrafts_.insert(currentConversation_, draft);
            }
        }
        updateConversationAvailability();
    });
    connect(send_, &QPushButton::clicked, this, &MainWindow::sendMessage);
    connect(taskDetails, &QPushButton::clicked, this, &MainWindow::conversationTaskDetails);
    connect(cancelTask_, &QPushButton::clicked, this, &MainWindow::cancelConversationTask);
    updateConversationAvailability();
    return page;
}

void MainWindow::updateConversationAvailability()
{
    if (!composer_)
        return;
    const auto content = composer_->toPlainText().trimmed();
    const bool saved = !conversation_.isEmpty();
    const bool active = saved && conversation_.value("state").toString() != "ARCHIVED";
    bool pendingTask = false;
    for (const auto &value : conversationBlockers_) {
        const auto row = value.toObject();
        pendingTask |= row.value("kind") == "task" && row.value("state") == "WAITING_APPROVAL";
    }
    if (!pendingTask) {
        for (const auto &value : conversationTasks_)
            pendingTask |= value.toObject().value("state") == "WAITING_APPROVAL" ||
                           value.toObject().value("state") == "PROPOSED";
    }
    const bool canDraft = conversationDrafts_.contains(currentConversation_) || conversationDrafts_.size() < draftLimit;
    composer_->setReadOnly(!active || !canDraft);
    composer_->setToolTip(canDraft ? QString{} : tr("Clear another draft to start editing this conversation."));
    send_->setEnabled(online_ && active && !content.isEmpty() && content.toUtf8().size() <= draftBytes
                      && !conversationSending_.contains(currentConversation_)
                      && !conversationDeletes_.contains(currentConversation_));
    send_->setToolTip(content.toUtf8().size() > draftBytes ? tr("Enter a message of up to 8 KiB.")
        : !online_ ? tr("Connect the runtime to send.") : tr("Send (Ctrl+Enter)"));
    conversationActions_->setVisible(saved);
    conversationActions_->setEnabled(online_ && active);
    cancelTask_->setVisible(saved && (!conversationTasks_.isEmpty() || pendingTask));
    cancelTask_->setEnabled(online_ && active && pendingTask);
    olderMessages_->setVisible(saved && hasCursor(transcriptBefore_));
    olderMessages_->setEnabled(online_ && !transcriptLoading_);
    latestMessages_->setVisible(saved && transcriptDetached_);
    latestMessages_->setEnabled(online_ && !transcriptLoading_);
    auto *panel = composer_->parentWidget();
    panel->findChild<QPushButton *>("clearDraft")->setEnabled(active && !composer_->document()->isEmpty()
        && !conversationSending_.contains(currentConversation_));
    panel->findChild<QPushButton *>("restoreDraft")->setVisible(active && composer_->document()->isEmpty()
        && clearedDrafts_.contains(currentConversation_));
    if (auto *start = composer_->parentWidget()->findChild<QPushButton *>("emptyNewChat"))
        start->setVisible(!saved);
    if (auto *details = composer_->parentWidget()->findChild<QPushButton *>("conversationTaskDetails"))
        details->setVisible(saved);
}

bool MainWindow::canLoadMoreConversations() const
{
    return online_ && !conversationLoading_ && hasCursor(conversationCursor_) &&
           conversationList_->property("loadedQuery").toString() == conversationSearch_->text().trimmed();
}

void MainWindow::updateConversationSelection()
{
    auto *selection = conversationList_->selectionModel();
    if (pages_->currentWidget() == conversationsPage_ && !currentConversation_.isEmpty()) {
        for (int row = 0; row < conversationList_->count(); ++row) {
            if (conversationList_->item(row)->data(IdRole).toString() == currentConversation_) {
                const auto index = conversationList_->model()->index(row, 0);
                selection->select(index, QItemSelectionModel::ClearAndSelect);
                return;
            }
        }
    }
    selection->clearSelection();
}

void MainWindow::refreshConversations(bool append)
{
    if (!online_ || !conversationSearch_ || (append && !canLoadMoreConversations()))
        return;
    if (conversationListInFlight_) {
        if (!append)
            conversationRefreshQueued_ = true;
        return;
    }
    const auto query = conversationSearch_->text().trimmed();
    QJsonObject params{{"limit", conversationPageSize}, {"query", query}};
    if (append)
        params.insert("before", conversationCursor_);
    const auto request = ++conversationRequest_;
    conversationLoading_ = true;
    conversationListInFlight_ = true;
    auto *state = conversationList_->parentWidget()->findChild<QLabel *>("conversationHistoryState");
    if (conversationList_->count() == 0) {
        state->setText(tr("Loading conversations…"));
        state->show();
    }
    call("conversation.list", params, [this, request, append, query, state](const QJsonObject &data, const QString &error) {
        conversationListInFlight_ = false;
        const bool refreshQueued = std::exchange(conversationRefreshQueued_, false);
        if (request != conversationRequest_ || refreshQueued) {
            if (refreshQueued)
                refreshConversations();
            return;
        }
        if (!error.isEmpty()) {
            conversationLoading_ = false;
            state->setText(tr("Conversation history could not load."));
            state->show();
            showStatus(error, true);
            return;
        }
        auto *scrollbar = conversationList_->verticalScrollBar();
        const QSignalBlocker listSignals(conversationList_);
        const auto scroll = scrollbar->value();
        const auto *focusedRow = conversationList_->currentItem();
        const auto focusedId = focusedRow ? focusedRow->data(IdRole).toString() : QString{};
        const auto firstPage = data.value("items").toArray();
        auto records = firstPage;
        const bool keepTail = !append && !firstPage.isEmpty()
            && conversationList_->property("loadedQuery").toString() == query
            && conversationList_->property("historyPaged").toBool();
        if (keepTail) {
            QSet<QString> freshIds;
            for (const auto &value : firstPage)
                freshIds.insert(value.toObject().value("id").toString());
            for (int i = 0; i < conversationList_->count(); ++i) {
                const auto *row = conversationList_->item(i);
                if (!freshIds.contains(row->data(IdRole).toString()))
                    records.append(row->data(RecordRole).toJsonObject());
            }
        }
        if (!append && conversationList_->property("loadedQuery").toString() != query)
            conversationList_->setProperty("historyWindowMoved", false);
        conversationList_->setProperty("loadedQuery", query);
        conversationList_->setProperty("historyPaged", append || keepTail);
        QHash<QString, QListWidgetItem *> existing;
        for (int index = 0; index < conversationList_->count(); ++index) {
            auto *item = conversationList_->item(index);
            existing.insert(item->data(IdRole).toString(), item);
        }
        int rowIndex = append ? conversationList_->count() : 0;
        for (const auto &value : records) {
            const auto row = value.toObject();
            const auto title = row.value("title").toString();
            const auto status = row.value("state").toString();
            const auto id = row.value("id").toString();
            const auto updated = row.value("updated_at").toInteger();
            QString age;
            if (updated > 0) {
                const auto seconds = qMax<qint64>(0, QDateTime::currentSecsSinceEpoch() - updated);
                age = seconds < 60 ? tr("now") : seconds < 3600 ? tr("%1m").arg(seconds / 60)
                    : seconds < 86400 ? tr("%1h").arg(seconds / 3600) : tr("%1d").arg(seconds / 86400);
            }
            auto *item = existing.value(id);
            if (append && item)
                continue;
            if (!item) {
                item = new QListWidgetItem;
                conversationList_->insertItem(rowIndex, item);
            } else if (const auto previous = conversationList_->row(item); previous != rowIndex) {
                conversationList_->insertItem(rowIndex, conversationList_->takeItem(previous));
            }
            const auto caption = age.isEmpty() ? title : tr("%1 · %2").arg(title, age);
            if (item->text() != caption)
                item->setText(caption);
            if (item->data(RecordRole).toJsonObject() != row) {
                item->setToolTip((title + QStringLiteral("\n") + status).toHtmlEscaped());
                item->setData(IdRole, id);
                item->setData(RecordRole, row);
                item->setData(Qt::AccessibleTextRole, tr("%1 · %2").arg(title, status));
            }
            if (!append && !focusedId.isEmpty() && id == focusedId)
                conversationList_->setCurrentItem(item, QItemSelectionModel::NoUpdate);
            ++rowIndex;
        }
        while (conversationList_->count() > rowIndex)
            delete conversationList_->takeItem(conversationList_->count() - 1);
        const int excess = qMax(0, conversationList_->count() - conversationCacheLimit);
        for (int index = 0; index < excess; ++index)
            delete conversationList_->takeItem(append ? 0 : conversationList_->count() - 1);
        if (append && excess)
            conversationList_->setProperty("historyWindowMoved", true);
        updateConversationSelection();
        scrollbar->setValue(append ? qMax(0, scroll - excess) : scroll);
        if (!keepTail) {
            const auto next = data.value("next_before");
            conversationCursor_ = firstPage.size() == conversationPageSize && (!append || next != conversationCursor_)
                ? next : QJsonValue{};
        } else if (excess) {
            conversationCursor_ = conversationList_->item(conversationList_->count() - 1)->data(IdRole).toString();
        }
        state->setText(conversationSearch_->text().trimmed().isEmpty() ? tr("No conversations yet.")
                                                                      : tr("No matching conversations."));
        state->setVisible(conversationList_->count() == 0);
        conversationLoading_ = false;
    });
}

void MainWindow::newConversation(const QString &preselected)
{
    if (!online_) {
        showStatus(tr("Connect the runtime to start a conversation."), true);
        return;
    }
    if (auto *existing = findChild<QDialog *>("newConversationDialog", Qt::FindDirectChildrenOnly)) {
        reactivate(existing);
        return;
    }
    auto *dialog = new QDialog(this);
    dialog->setObjectName("newConversationDialog");
    dialog->setAttribute(Qt::WA_DeleteOnClose);
    dialog->setWindowTitle(tr("New Chat"));
    const auto *screen = window()->screen();
    dialog->resize(screen ? QSize(570, 390).boundedTo(screen->availableGeometry().size() - QSize(32, 72))
                          : QSize(570, 390));
    auto *layout = new QVBoxLayout(dialog);
    auto *fields = new QFormLayout;
    auto *title = new QLineEdit(tr("New conversation"));
    title->setObjectName("newConversationTitle");
    title->selectAll();
    title->setMaxLength(120);
    fields->addRow(tr("Title"), title);
    fields->addRow(tr("Agent profile"), label(tr("Local handoff · native Agent execution unavailable"), "meta"));
    layout->addLayout(fields);
    auto *picker = new RepositoryPicker(true, preselected);
    layout->addWidget(picker, 1);
    auto *status = label({}, "meta");
    status->hide();
    layout->addWidget(status);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
    auto *ok = buttons->button(QDialogButtonBox::Ok);
    auto *cancel = buttons->button(QDialogButtonBox::Cancel);
    ok->setText(tr("Create chat"));
    cancel->setText(tr("Cancel"));
    layout->addWidget(buttons);

    const auto enable = [ok, cancel, picker, title, dialog] {
        const bool saving = dialog->property("saving").toBool();
        cancel->setEnabled(!saving);
        picker->manageButton()->setEnabled(!saving);
        ok->setEnabled(!saving && !picker->selection().isEmpty()
                       && !title->text().trimmed().isEmpty());
    };
    const QPointer<QDialog> guard(dialog);
    const auto load = [this, guard, picker, enable] {
        ensureRepositoryCatalog([this, guard, picker, enable](bool loaded) {
            if (!guard)
                return;
            picker->populate(repositories_, loaded);
            enable();
        });
    };
    connect(picker->list(), &QListWidget::itemSelectionChanged, dialog, enable);
    connect(title, &QLineEdit::textChanged, dialog, enable);
    connect(buttons, &QDialogButtonBox::rejected, dialog, &QDialog::reject);
    connect(dialog, &QDialog::finished, dialog, [dialog] { dialog->setProperty("finished", true); });
    connect(picker->manageButton(), &QPushButton::clicked, dialog, [this, dialog] {
        if (auto *origin = pages_->currentWidget(); origin != settingsPage_) {
            dialog->setProperty("repositoryReturnPage", QVariant::fromValue(origin));
            dialog->hide();
        }
        showSettings(QStringLiteral("repositories"));
    });
    connect(pages_, &QStackedWidget::currentChanged, dialog, [this, dialog, load](int) {
        if (pages_->currentWidget() == dialog->property("repositoryReturnPage").value<QWidget *>()) {
            dialog->setProperty("repositoryReturnPage", {});
            reactivate(dialog);
            load();
        }
    });
    connect(buttons, &QDialogButtonBox::accepted, dialog, [this, guard, title, picker, status, enable] {
        if (!guard || guard->property("saving").toBool())
            return;
        if (!online_) {
            status->setText(tr("Reconnect the local runtime to save."));
            status->show();
            return;
        }
        const QJsonObject payload{{"title", title->text().trimmed()}, {"bindings", picker->selection()},
                                  {"profile", "handoff"}};
        if (guard->property("submittedPayload").toJsonObject() != payload) {
            guard->setProperty("submittedPayload", payload);
            guard->setProperty("submissionKey", requestKey());
        }
        auto params = payload;
        params.insert("key", guard->property("submissionKey").toString());
        guard->setProperty("saving", true);
        enable();
        status->setText(tr("Saving conversation…"));
        status->show();
        const auto navigation = navigationRevision_;
        call("conversation.create", params, [this, guard, status, enable, navigation](const QJsonObject &result, const QString &error) {
            if (!guard || guard->property("finished").toBool()) {
                if (!error.isEmpty())
                    showStatus(error, true);
                else
                    refreshConversations();
                return;
            }
            guard->setProperty("saving", false);
            enable();
            const auto id = result.value("id").toString();
            if (!error.isEmpty() || id.isEmpty()) {
                status->setText(error.isEmpty() ? tr("The runtime did not return the saved conversation.") : error);
                return;
            }
            guard->accept();
            if (navigation == navigationRevision_)
                navigateConversation(id);
            refreshConversations();
        });
    });
    enable();
    reactivate(dialog);
    title->setFocus();
    load();
}

void MainWindow::navigateConversation(const QString &id)
{
    if (!online_ || id.isEmpty())
        return;
    if (id != currentConversation_ || conversation_.isEmpty())
        openConversation(id);
    showPage(conversationsPage_);
}

void MainWindow::openConversation(const QString &id, TranscriptPage page)
{
    if (!online_ || id.isEmpty())
        return;
    if (id != currentConversation_) {
        ++navigationRevision_;
        currentConversation_ = id;
        eventCursor_ = 0;
        conversation_ = {};
        conversationTasks_ = {};
        conversationBindings_ = {};
        conversationBlockers_ = {};
        transcriptRows_.clear();
        transcriptBefore_ = {};
        transcriptNewest_ = 0;
        transcriptDetached_ = false;
        transcriptRefreshQueued_ = false;
        const QSignalBlocker guard(composer_);
        composer_->setPlainText(conversationDrafts_.value(id));
        transcript_->clear();
        conversationTitle_->setText(tr("Chat"));
        conversationDetail_->setText(tr("Loading conversation…"));
        updateConversationAvailability();
    }
    if (page != TranscriptPage::Latest && transcriptLoading_) {
        transcriptRefreshQueued_ |= page == TranscriptPage::Newer;
        return;
    }
    QJsonObject params{{"id", id}};
    if (page == TranscriptPage::Older) {
        if (!hasCursor(transcriptBefore_))
            return;
        params.insert("before", transcriptBefore_);
    } else if (page == TranscriptPage::Newer) {
        params.insert("after", QString::number(transcriptNewest_));
    }
    const auto request = ++detailRequest_;
    transcriptLoading_ = true;
    updateConversationAvailability();
    call("conversation.get", params, [this, id, request, page](const QJsonObject &result, const QString &error) {
        if (request != detailRequest_ || currentConversation_ != id)
            return;
        transcriptLoading_ = false;
        const bool queued = std::exchange(transcriptRefreshQueued_, false);
        if (!error.isEmpty()) {
            if (result.value("code") == "NOT_FOUND") {
                forgetConversation(id, true);
                return;
            }
            conversationDetail_->setText(error);
            updateConversationAvailability();
            showStatus(error, true);
            return;
        }
        conversation_ = result.value("conversation").toObject();
        const auto tasks = result.value("tasks").toArray();
        if (page == TranscriptPage::Latest || (page == TranscriptPage::Newer && !tasks.isEmpty()))
            conversationTasks_ = tasks;
        conversationBindings_ = result.value("bindings").toArray();
        conversationBlockers_ = result.value("blockers").toArray();
        conversationTitle_->setText(conversation_.value("title").toString(tr("Conversation")));
        applyTranscriptPage(result, page);
        updateConversationAvailability();
        updateConversationSummary();
        if (queued || (page == TranscriptPage::Newer && result.value("has_more").toBool()
                       && !result.value("messages").toArray().isEmpty()))
            QTimer::singleShot(0, this, [this, id] {
                if (currentConversation_ == id && pages_->currentWidget() == conversationsPage_)
                    openConversation(id, TranscriptPage::Newer);
            });
    });
}

void MainWindow::applyTranscriptPage(const QJsonObject &data, TranscriptPage page)
{
    QList<QJsonObject> incoming;
    for (const auto &value : data.value("messages").toArray()) {
        const auto row = value.toObject();
        if (page != TranscriptPage::Older)
            transcriptNewest_ = qMax(transcriptNewest_, row.value("ordinal").toInteger());
        if (page == TranscriptPage::Latest || !transcriptRows_.contains(row))
            incoming.append(row);
    }
    if (page != TranscriptPage::Newer)
        transcriptBefore_ = data.value("next_before");
    if (page == TranscriptPage::Newer && (incoming.isEmpty() || transcriptDetached_))
        return;
    if (page == TranscriptPage::Older && incoming.isEmpty())
        return;
    if (page == TranscriptPage::Latest && incoming == transcriptRows_) {
        transcriptDetached_ = false;
        return;
    }
    auto *scrollbar = transcript_->verticalScrollBar();
    const auto previous = scrollbar->value();
    const auto maximum = scrollbar->maximum();
    const auto textFor = [this](const QList<QJsonObject> &rows) {
        QStringList parts;
        for (const auto &row : rows)
            parts.append(displayedMessage(row, tr("You"), tr("Display shortened.")));
        return parts.join(QStringLiteral("\n\n"));
    };
    const bool older = page == TranscriptPage::Older;
    const bool replacing = page == TranscriptPage::Latest;
    if (replacing) {
        transcriptRows_ = incoming;
        transcriptDetached_ = false;
    } else if (older) {
        transcriptRows_ = incoming + transcriptRows_;
    } else {
        transcriptRows_.append(incoming);
    }
    qsizetype blocks = 0;
    for (const auto &row : transcriptRows_)
        blocks += qMin<qsizetype>(400, row.value("content").toString().count(u'\n')) + 4;
    bool trimmed = false;
    while (transcriptRows_.size() > transcriptLimit || blocks > transcriptBlockLimit) {
        const auto removed = older ? transcriptRows_.takeLast() : transcriptRows_.takeFirst();
        blocks -= qMin<qsizetype>(400, removed.value("content").toString().count(u'\n')) + 4;
        trimmed = true;
    }
    if (trimmed && older)
        transcriptDetached_ = true;
    if (trimmed && !older && !transcriptRows_.isEmpty())
        transcriptBefore_ = QString::number(transcriptRows_.first().value("ordinal").toInteger());
    if (replacing || trimmed) {
        transcript_->setPlainText(textFor(transcriptRows_));
    } else {
        QTextCursor cursor(transcript_->document());
        cursor.movePosition(older ? QTextCursor::Start : QTextCursor::End);
        const auto text = textFor(incoming);
        cursor.insertText(older ? text + QStringLiteral("\n\n")
            : (transcript_->document()->isEmpty() ? QString{} : QStringLiteral("\n\n")) + text);
    }
    if (older) {
        scrollbar->setValue(previous + qMax(0, scrollbar->maximum() - maximum));
    } else if (previous >= maximum) {
        transcript_->moveCursor(QTextCursor::End);
    } else {
        scrollbar->setValue(previous);
    }
}

void MainWindow::updateConversationSummary()
{
    if (conversation_.isEmpty())
        return;
    QStringList bindings;
    for (const auto &value : conversationBindings_) {
        const auto id = value.toObject().value("binding_id").toString();
        QString name = id;
        for (const auto &entry : repositories_) {
            const auto row = entry.toObject();
            if (row.value("binding_id").toString() != id)
                continue;
            name = tr("%1 · %2").arg(row.value("display_name").toString(), row.value("branch").toString(tr("Detached")));
            break;
        }
        bindings.append(name);
    }
    QString detail = tr("Local handoff · %1").arg(conversation_.value("state").toString());
    if (!bindings.isEmpty())
        detail += QStringLiteral(" · ") + bindings.join(QStringLiteral(", "));
    const auto state = conversationTasks_.isEmpty() ? QString() : conversationTasks_.first().toObject().value("state").toString();
    if (!state.isEmpty())
        detail += QStringLiteral("\n") + tr("Latest task: %1").arg(state);
    for (const auto &value : conversationBlockers_) {
        const auto blocker = value.toObject();
        detail += QStringLiteral("\n") + tr("%1 %2 · %3")
            .arg(blocker.value("kind").toString(), blocker.value("id").toString(), blocker.value("state").toString());
    }
    conversationDetail_->setText(detail);
}

void MainWindow::conversationTaskDetails()
{
    if (conversation_.isEmpty()) {
        showStatus(tr("Select a conversation to inspect its task details."));
        return;
    }
    QDialog dialog(this);
    dialog.setWindowTitle(tr("Task details"));
    dialog.resize(720, 460);
    auto *layout = new QVBoxLayout(&dialog);
    auto *description = label(tr("Persisted local intake, repository bindings and routing state."), "lead");
    layout->addWidget(description);
    auto *details = new QPlainTextEdit(QString::fromUtf8(QJsonDocument(
        QJsonObject{{"bindings", conversationBindings_}, {"tasks", conversationTasks_}, {"blockers", conversationBlockers_}})
        .toJson(QJsonDocument::Indented)));
    details->setProperty("codeText", true);
    details->setAccessibleName(tr("Task and route details"));
    details->setReadOnly(true);
    layout->addWidget(details, 1);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close);
    buttons->button(QDialogButtonBox::Close)->setText(tr("Close"));
    layout->addWidget(buttons);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    dialog.exec();
}

void MainWindow::cancelConversationTask()
{
    if (!online_)
        return;
    const auto id = currentConversation_;
    QString task;
    for (const auto &value : conversationBlockers_) {
        const auto blocker = value.toObject();
        if (blocker.value("kind") == "task" && blocker.value("state") == "WAITING_APPROVAL") {
            task = blocker.value("id").toString();
            break;
        }
    }
    if (task.isEmpty()) {
        for (const auto &value : conversationTasks_) {
            const auto candidate = value.toObject();
            if (candidate.value("state") == "WAITING_APPROVAL" || candidate.value("state") == "PROPOSED") {
                task = candidate.value("id").toString();
                break;
            }
        }
    }
    if (task.isEmpty())
        return;
    cancelTask_->setEnabled(false);
    call("conversation.cancel", {{"id", id}, {"task_id", task}},
         [this, id](const QJsonObject &, const QString &error) {
        if (!error.isEmpty())
            showError(error);
        if (currentConversation_ == id)
            openConversation(id);
    });
}

void MainWindow::editConversation(bool archive, const QJsonObject &record)
{
    const auto target = record.isEmpty() ? conversation_ : record;
    if (!online_ || target.isEmpty() || target.value("state").toString() == "ARCHIVED")
        return;
    const auto id = target.value("id").toString();
    QJsonObject params{{"id", id}, {"revision", target.value("revision")}};
    if (archive) {
        QMessageBox confirmation(QMessageBox::Question, tr("Archive conversation"),
            tr("Archive this conversation and keep its saved history?"),
            QMessageBox::Yes | QMessageBox::Cancel, this);
        confirmation.button(QMessageBox::Yes)->setText(tr("Archive"));
        confirmation.button(QMessageBox::Cancel)->setText(tr("Cancel"));
        confirmation.setDefaultButton(QMessageBox::Cancel);
        if (confirmation.exec() != QMessageBox::Yes)
            return;
    } else {
        QInputDialog rename(this);
        rename.setWindowTitle(tr("Rename conversation"));
        rename.setLabelText(tr("Title"));
        rename.setOkButtonText(tr("Rename"));
        rename.setCancelButtonText(tr("Cancel"));
        rename.setTextValue(target.value("title").toString());
        if (rename.exec() != QDialog::Accepted || rename.textValue().trimmed().isEmpty())
            return;
        params.insert("title", rename.textValue().trimmed());
    }
    call(archive ? "conversation.archive" : "conversation.rename", params,
         [this, id, archive](const QJsonObject &updated, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        if (archive) {
            forgetConversation(id, false);
            showStatus(tr("Conversation archived."));
            return;
        }
        for (int index = 0; index < conversationList_->count(); ++index) {
            auto *item = conversationList_->item(index);
            if (item->data(IdRole).toString() == id && updated.value("id").toString() == id) {
                const auto query = conversationSearch_->text().trimmed();
                if (updated.value("title").toString().contains(query, Qt::CaseInsensitive))
                    item->setData(RecordRole, updated);
                else
                    delete conversationList_->takeItem(index);
                break;
            }
        }
        if (id == currentConversation_)
            openConversation(id);
        refreshConversations();
    });
}

void MainWindow::clearCurrentDraft()
{
    const auto id = currentConversation_;
    if (id.isEmpty() || composer_->isReadOnly() || conversationSending_.contains(id))
        return;
    if (clearedDrafts_.size() >= draftLimit && !clearedDrafts_.contains(id)) {
        showError(tr("The undo buffer for cleared drafts is full."));
        return;
    }
    clearedDrafts_.insert(id, composer_->toPlainText());
    composer_->clear();
    composer_->setFocus();
}

void MainWindow::restoreCurrentDraft()
{
    const auto id = currentConversation_;
    if (id.isEmpty() || !composer_->document()->isEmpty() || composer_->isReadOnly())
        return;
    const auto draft = clearedDrafts_.take(id);
    if (draft.isEmpty())
        return;
    composer_->setPlainText(draft);
    composer_->moveCursor(QTextCursor::End);
    composer_->setFocus();
}

void MainWindow::forgetConversation(const QString &id, bool deleted)
{
    if (deleted) {
        conversationDrafts_.remove(id);
        conversationRetryKeys_.remove(id);
        conversationRetryContents_.remove(id);
        clearedDrafts_.remove(id);
    }
    ++conversationRequest_;
    for (int index = 0; index < conversationList_->count(); ++index) {
        if (conversationList_->item(index)->data(IdRole).toString() == id) {
            delete conversationList_->takeItem(index);
            break;
        }
    }
    if (id == currentConversation_) {
        ++detailRequest_;
        ++navigationRevision_;
        currentConversation_.clear();
        conversation_ = {};
        conversationTasks_ = {};
        conversationBindings_ = {};
        conversationBlockers_ = {};
        transcriptRows_.clear();
        transcriptBefore_ = {};
        transcriptNewest_ = 0;
        transcriptLoading_ = false;
        transcriptDetached_ = false;
        transcriptRefreshQueued_ = false;
        eventCursor_ = 0;
        composer_->clear();
        transcript_->clear();
        conversationTitle_->setText(tr("Chat"));
        conversationDetail_->setText(tr("Select a saved conversation or start a new chat."));
        updateConversationAvailability();
        updateConversationSelection();
    }
    refreshConversations();
}

void MainWindow::deleteConversationData(const QString &id, const QString &messageId,
                                        std::function<void(bool)> onComplete)
{
    if (!online_ || id.isEmpty() || conversationSending_.contains(id) ||
        conversationDeletes_.contains(id) || conversationDeletes_.size() >= 8) {
        if (onComplete)
            onComplete(false);
        showError(tr("The conversation cannot be deleted while another change is pending."));
        return;
    }
    conversationDeletes_.insert(id);
    updateConversationAvailability();
    const auto finish = [this, id, onComplete = std::move(onComplete)](bool success) {
        conversationDeletes_.remove(id);
        updateConversationAvailability();
        if (onComplete)
            onComplete(success);
    };
    QJsonObject params{{"id", id}};
    if (!messageId.isEmpty())
        params.insert("message_id", messageId);
    const auto method = messageId.isEmpty() ? QStringLiteral("conversation.delete") : QStringLiteral("message.delete");
    const auto navigation = navigationRevision_;
    const auto section = pages_->currentWidget() == settingsPage_ ? settingsSection() : QString{};
    call(method + QStringLiteral(".preview"), params,
         [this, params, id, messageId, method, finish, navigation, section](const QJsonObject &preview, const QString &error) {
        if (navigation != navigationRevision_ || (!section.isEmpty() && section != settingsSection())) {
            finish(false);
            return;
        }
        if (!error.isEmpty()) {
            finish(false);
            showError(error);
            return;
        }
        if (preview.value("blocker_count").toInteger() > 0) {
            QStringList blockers;
            for (const auto &value : preview.value("blockers").toArray()) {
                const auto row = value.toObject();
                blockers.append(tr("%1 %2 · %3").arg(row.value("kind").toString(), row.value("id").toString(),
                                                      row.value("state").toString()));
            }
            finish(false);
            auto *notice = new QMessageBox(QMessageBox::Warning, tr("Deletion blocked"),
                tr("Resolve the active tasks or runs before retrying.\n%1").arg(blockers.join(u'\n')),
                QMessageBox::Close, this);
            notice->setTextFormat(Qt::PlainText);
            notice->setAttribute(Qt::WA_DeleteOnClose);
            notice->open();
            return;
        }
        const auto title = messageId.isEmpty() ? tr("Delete conversation") : tr("Delete submitted message");
        const auto target = messageId.isEmpty() ? tr("Conversation %1").arg(id)
            : tr("Message %1 in conversation %2").arg(messageId, id);
        auto description =
            tr("%1 · %2\nDelete %3 local message(s) and %4 task record(s)? Existing backups and provider copies may "
               "retain content. Repository files are kept.")
                .arg(target, preview.value("title").toString()).arg(preview.value("message_count").toInteger())
                .arg(preview.value("task_count").toInteger());
        QStringList automationIds;
        if (messageId.isEmpty()) {
            QStringList automationLabels;
            for (const auto &value : preview.value("automations").toArray()) {
                const auto row = value.toObject();
                const auto automationId = row.value("id").toString();
                if (automationId.isEmpty())
                    continue;
                automationIds.append(automationId);
                automationLabels.append(tr("Automation %1 · %2").arg(automationId, row.value("name").toString()));
            }
            if (!automationIds.isEmpty()) {
                description += QStringLiteral("\n\n") +
                    tr("Delete %1 linked automation(s) and stop their schedules. Terminal run history is retained.")
                        .arg(automationIds.size()) + QStringLiteral("\n") + automationLabels.join(u'\n');
            }
        }
        auto *confirm = new QMessageBox(QMessageBox::Warning, title, description,
                                         QMessageBox::Yes | QMessageBox::Cancel, this);
        confirm->setObjectName("deleteDataConfirmation");
        confirm->setTextFormat(Qt::PlainText);
        confirm->button(QMessageBox::Yes)->setText(tr("Delete permanently"));
        confirm->setDefaultButton(QMessageBox::Cancel);
        confirm->setAttribute(Qt::WA_DeleteOnClose);
        connect(confirm, &QMessageBox::finished, this,
                [this, id, messageId, method, params, preview, finish, automationIds](int answer) {
            if (answer != QMessageBox::Yes) {
                finish(false);
                return;
            }
            auto mutation = params;
            mutation.insert("key", requestKey());
            mutation.insert("revision", preview.value("revision"));
            mutation.insert("token", preview.value("token"));
            call(method, mutation, [this, id, messageId, finish, automationIds](const QJsonObject &, const QString &failure) {
                if (!failure.isEmpty()) {
                    finish(false);
                    showError(failure);
                    return;
                }
                if (messageId.isEmpty()) {
                    forgetConversation(id, true);
                    forgetAutomations(automationIds);
                } else if (id == currentConversation_) {
                    transcriptRows_.clear();
                    transcript_->clear();
                    openConversation(id);
                }
                finish(true);
                showStatus(messageId.isEmpty() ? tr("Conversation deleted.") : tr("Submitted message deleted."));
            });
        });
        confirm->open();
    });
}

void MainWindow::sendMessage()
{
    if (!send_->isEnabled())
        return;
    const auto id = currentConversation_;
    const auto content = composer_->toPlainText().trimmed();
    if (content != conversationRetryContents_.value(id) || conversationRetryKeys_.value(id).isEmpty()) {
        conversationRetryContents_[id] = content;
        conversationRetryKeys_[id] = requestKey();
    }
    conversationSending_.insert(id);
    updateConversationAvailability();
    QJsonObject params{{"id", id}, {"content", content}, {"key", conversationRetryKeys_.value(id)}};
    if (conversation_.value("revision").toInteger() > 0)
        params.insert("revision", conversation_.value("revision"));
    call("conversation.send", params, [this, id, content](const QJsonObject &result, const QString &error) {
        conversationSending_.remove(id);
        if (!error.isEmpty()) {
            updateConversationAvailability();
            showStatus(error, true);
            if (id == currentConversation_ && result.value("code") == "CONFLICT")
                openConversation(id);
            return;
        }
        conversationRetryContents_.remove(id);
        conversationRetryKeys_.remove(id);
        if (conversationDrafts_.value(id).trimmed() == content) {
            conversationDrafts_.remove(id);
            if (id == currentConversation_) {
                const QSignalBlocker guard(composer_);
                composer_->clear();
            }
        }
        if (id == currentConversation_) {
            if (result.value("conversation_revision").isDouble())
                conversation_.insert("revision", result.value("conversation_revision"));
            updateConversationAvailability();
            openConversation(id, TranscriptPage::Newer);
        }
        showStatus(tr("Message saved. Review task status in the chat."));
        refreshConversations();
    });
}

void MainWindow::refreshConversationEvents()
{
    if (!online_ || eventRequestPending_ || currentConversation_.isEmpty()
        || pages_->currentWidget() != conversationsPage_)
        return;
    const auto id = currentConversation_;
    eventRequestPending_ = true;
    call("conversation.events", {{"id", id}, {"after", QString::number(eventCursor_)}, {"limit", 30}},
         [this, id](const QJsonObject &result, const QString &error) {
        eventRequestPending_ = false;
        if (id != currentConversation_ || pages_->currentWidget() != conversationsPage_)
            return;
        if (!error.isEmpty()) {
            if (result.value("code") == "NOT_FOUND")
                forgetConversation(id, true);
            return;
        }
        const auto cursor = result.value("next_after");
        const auto next = cursor.isString() ? cursor.toString().toULongLong() : quint64(qMax<qint64>(0, cursor.toInteger()));
        if (next > eventCursor_) {
            eventCursor_ = next;
            bool reset = false;
            for (const auto &value : result.value("items").toArray()) {
                const auto type = value.toObject().value("type").toString();
                reset |= type == "MESSAGE_DELETED" || type == "CONVERSATION_ARCHIVED" || type == "PROPOSAL_CANCELLED";
            }
            openConversation(id, reset ? TranscriptPage::Latest : TranscriptPage::Newer);
        }
    });
}
