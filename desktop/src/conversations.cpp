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
#include <QVBoxLayout>

namespace {
constexpr int IdRole = Qt::UserRole;
constexpr int RecordRole = Qt::UserRole + 1;
constexpr int conversationPageSize = 30;

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
        showPage(conversationsPage_);
        openConversation(item->data(IdRole).toString());
    });
    connect(conversationList_, &QListWidget::itemClicked, this, [this](QListWidgetItem *item) {
        showPage(conversationsPage_);
        if (currentConversation_ != item->data(IdRole).toString())
            openConversation(item->data(IdRole).toString());
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
        if (row) {
            const bool active = online_ && record.value("state").toString() != "ARCHIVED";
            resume = menu.addAction(tr("Open conversation"));
            rename = menu.addAction(tr("Rename"));
            archive = menu.addAction(tr("Archive"));
            resume->setEnabled(online_);
            rename->setEnabled(active);
            archive->setEnabled(active);
        }
        QAction *more = nullptr;
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
        } else if (choice == resume) {
            showPage(conversationsPage_);
            openConversation(record.value("id").toString());
        } else if (choice == rename || choice == archive) {
            editConversation(choice == archive, record);
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
    connect(rename, &QAction::triggered, this, [this] { editConversation(false); });
    connect(archive, &QAction::triggered, this, [this] { editConversation(true); });
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
    actions->addStretch();
    layout->addLayout(actions);

    transcript_ = new QPlainTextEdit;
    transcript_->setObjectName("conversationTranscript");
    transcript_->setReadOnly(true);
    transcript_->setAccessibleName(tr("Conversation transcript"));
    transcript_->setPlaceholderText(tr("Saved messages and task outcomes appear here."));
    transcript_->setMaximumBlockCount(3500);
    layout->addWidget(transcript_, 1);
    composer_ = new CompactTextEdit(5);
    composer_->setObjectName("conversationComposer");
    composer_->setAccessibleName(tr("Message"));
    composer_->setPlaceholderText(tr("Describe a scoped task…"));
    layout->addWidget(composer_);
    auto *commands = new QHBoxLayout;
    commands->addWidget(label(tr("Local handoff · Ctrl+Enter to send"), "meta"), 1);
    send_ = new QPushButton(tr("Send"));
    send_->setObjectName("sendMessage");
    send_->setProperty("primary", true);
    commands->addWidget(send_);
    layout->addLayout(commands);

    auto *shortcut = new QShortcut(QKeySequence(Qt::CTRL | Qt::Key_Return), composer_);
    shortcut->setContext(Qt::WidgetWithChildrenShortcut);
    connect(shortcut, &QShortcut::activated, this, &MainWindow::sendMessage);
    connect(composer_, &QPlainTextEdit::textChanged, this, [this] {
        if (!currentConversation_.isEmpty())
            conversationDrafts_[currentConversation_] = composer_->toPlainText();
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
    const auto taskState = conversationTasks_.isEmpty()
        ? QString() : conversationTasks_.first().toObject().value("state").toString();
    composer_->setReadOnly(!active);
    send_->setEnabled(online_ && active && !content.isEmpty() && content.toUtf8().size() <= 8192
                      && !conversationSending_.contains(currentConversation_));
    send_->setToolTip(content.toUtf8().size() > 8192 ? tr("Enter a message of up to 8 KiB.")
        : !online_ ? tr("Connect the runtime to send.") : tr("Send (Ctrl+Enter)"));
    conversationActions_->setVisible(saved);
    conversationActions_->setEnabled(online_ && active);
    cancelTask_->setVisible(saved && !taskState.isEmpty());
    cancelTask_->setEnabled(online_ && active && (taskState == "PROPOSED" || taskState == "WAITING_APPROVAL"));
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

void MainWindow::refreshConversations(bool append)
{
    if (!online_ || !conversationSearch_ || (append && !canLoadMoreConversations()))
        return;
    const auto query = conversationSearch_->text().trimmed();
    QJsonObject params{{"limit", conversationPageSize}, {"query", query}};
    if (append)
        params.insert("before", conversationCursor_);
    const auto request = ++conversationRequest_;
    conversationLoading_ = true;
    auto *state = conversationList_->parentWidget()->findChild<QLabel *>("conversationHistoryState");
    if (conversationList_->count() == 0) {
        state->setText(tr("Loading conversations…"));
        state->show();
    }
    call("conversation.list", params, [this, request, append, query, state](const QJsonObject &data, const QString &error) {
        if (request != conversationRequest_)
            return;
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
        const auto selected = currentConversation_;
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
        if (!append) {
            conversationList_->clear();
            conversationList_->setProperty("loadedQuery", query);
        }
        conversationList_->setProperty("historyPaged", append || keepTail);
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
            auto *item = new QListWidgetItem(age.isEmpty() ? title : tr("%1 · %2").arg(title, age), conversationList_);
            item->setToolTip((title + QStringLiteral("\n") + status).toHtmlEscaped());
            item->setData(IdRole, id);
            item->setData(RecordRole, row);
            item->setData(Qt::AccessibleTextRole, tr("%1 · %2").arg(title, status));
            if (status == "ARCHIVED")
                item->setText(tr("Archived · %1").arg(item->text()));
            if (id == selected)
                conversationList_->setCurrentItem(item);
        }
        scrollbar->setValue(scroll);
        if (!keepTail) {
            const auto next = data.value("next_before");
            conversationCursor_ = firstPage.size() == conversationPageSize && (!append || next != conversationCursor_)
                ? next : QJsonValue{};
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
        dialog->setProperty("managingRepositories", true);
        dialog->hide();
        showSettings(QStringLiteral("repositories"));
    });
    connect(pages_, &QStackedWidget::currentChanged, dialog, [this, dialog, load](int) {
        if (pages_->currentWidget() == conversationsPage_ && dialog->property("managingRepositories").toBool()) {
            dialog->setProperty("managingRepositories", false);
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
        call("conversation.create", params, [this, guard, status, enable](const QJsonObject &result, const QString &error) {
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
            if (pages_->currentWidget() == conversationsPage_)
                openConversation(id);
            refreshConversations();
        });
    });
    enable();
    reactivate(dialog);
    title->setFocus();
    load();
}

void MainWindow::openConversation(const QString &id)
{
    if (!online_ || id.isEmpty())
        return;
    if (id != currentConversation_) {
        currentConversation_ = id;
        eventCursor_ = 0;
        conversation_ = {};
        conversationTasks_ = {};
        conversationBindings_ = {};
        const QSignalBlocker guard(composer_);
        composer_->setPlainText(conversationDrafts_.value(id));
        transcript_->clear();
        conversationTitle_->setText(tr("Chat"));
        conversationDetail_->setText(tr("Loading conversation…"));
        updateConversationAvailability();
    }
    const auto request = ++detailRequest_;
    call("conversation.get", {{"id", id}}, [this, id, request](const QJsonObject &result, const QString &error) {
        if (request != detailRequest_ || currentConversation_ != id)
            return;
        if (!error.isEmpty()) {
            conversationDetail_->setText(error);
            showStatus(error, true);
            return;
        }
        conversation_ = result.value("conversation").toObject();
        conversationTasks_ = result.value("tasks").toArray();
        conversationBindings_ = result.value("bindings").toArray();
        conversationTitle_->setText(conversation_.value("title").toString(tr("Conversation")));
        updateConversationAvailability();
        QStringList messages;
        for (const auto &value : result.value("messages").toArray()) {
            const auto row = value.toObject();
            const auto role = row.value("role").toString();
            messages.append((role == "USER" ? tr("You") : role) + "\n" + row.value("content").toString());
        }
        const auto text = messages.join(QStringLiteral("\n\n"));
        if (transcript_->toPlainText() != text) {
            const auto scroll = transcript_->verticalScrollBar()->value();
            const bool following = scroll >= transcript_->verticalScrollBar()->maximum();
            transcript_->setPlainText(text);
            if (following)
                transcript_->moveCursor(QTextCursor::End);
            else
                transcript_->verticalScrollBar()->setValue(scroll);
        }
        updateConversationSummary();
    });
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
        QJsonObject{{"bindings", conversationBindings_}, {"tasks", conversationTasks_}})
        .toJson(QJsonDocument::Indented)));
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
    if (!online_ || conversationTasks_.isEmpty())
        return;
    const auto id = currentConversation_;
    const auto task = conversationTasks_.first().toObject().value("id");
    if (task.toString().isEmpty())
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
         [this, id](const QJsonObject &updated, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
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
    call("conversation.send", {{"id", id}, {"content", content}, {"key", conversationRetryKeys_.value(id)}},
         [this, id, content](const QJsonObject &, const QString &error) {
        conversationSending_.remove(id);
        if (!error.isEmpty()) {
            updateConversationAvailability();
            showStatus(error, true);
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
            updateConversationAvailability();
            openConversation(id);
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
    call("conversation.events", {{"id", id}, {"after", qint64(eventCursor_)}, {"limit", 30}},
         [this, id](const QJsonObject &result, const QString &error) {
        eventRequestPending_ = false;
        if (!error.isEmpty() || id != currentConversation_)
            return;
        const auto next = quint64(qMax<qint64>(0, result.value("next_after").toInteger()));
        if (next > eventCursor_) {
            eventCursor_ = next;
            openConversation(id);
        }
    });
}
