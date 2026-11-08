#include "main_window.h"

#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMenu>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QSplitter>
#include <QStackedWidget>
#include <QTextCursor>
#include <QTimer>
#include <QUuid>
#include <QVBoxLayout>

namespace {
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
}

QWidget *MainWindow::makeConversations()
{
    auto *page = new QWidget;
    page->setObjectName("conversationsPage");
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(36, 31, 36, 24);
    layout->setSpacing(12);
    auto *top = new QHBoxLayout;
    top->addWidget(label(tr("Conversations"), "heading"), 1);
    auto *newChat = new QPushButton(tr("+  New Chat"));
    newChat->setProperty("primary", true);
    top->addWidget(newChat);
    layout->addLayout(top);
    layout->addWidget(label(tr("Saved sessions, scoped intake and task status."), "lead"));

    auto *split = new QSplitter;
    split->setChildrenCollapsible(false);
    auto *history = new QWidget;
    auto *historyLayout = new QVBoxLayout(history);
    historyLayout->setContentsMargins(0, 0, 7, 0);
    conversationSearch_ = new QLineEdit;
    conversationSearch_->setPlaceholderText(tr("Search conversations"));
    conversationSearch_->setAccessibleName(tr("Search conversations"));
    conversationList_ = new QListWidget;
    conversationList_->setObjectName("conversationList");
    conversationList_->setAccessibleName(tr("Conversation history"));
    conversationList_->setWordWrap(false);
    conversationList_->setUniformItemSizes(true);
    conversationMore_ = new QPushButton(tr("Load more"));
    conversationMore_->hide();
    historyLayout->addWidget(conversationSearch_);
    historyLayout->addWidget(conversationList_, 1);
    historyLayout->addWidget(conversationMore_);
    split->addWidget(history);

    auto *detail = new QWidget;
    auto *detailLayout = new QVBoxLayout(detail);
    detailLayout->setContentsMargins(8, 0, 0, 0);
    auto *detailHeader = new QHBoxLayout;
    conversationTitle_ = label(tr("Select a conversation"), "sectionTitle");
    conversationRename_ = new QPushButton(tr("Rename"));
    conversationArchive_ = new QPushButton(tr("Archive"));
    conversationRename_->setEnabled(false);
    conversationArchive_->setEnabled(false);
    detailHeader->addWidget(conversationTitle_, 1);
    detailHeader->addWidget(conversationRename_);
    detailHeader->addWidget(conversationArchive_);
    detailLayout->addLayout(detailHeader);
    conversationDetail_ = label(tr("The local runtime saves each accepted turn before displaying it."), "meta");
    conversationDetail_->setObjectName("conversationDetail");
    detailLayout->addWidget(conversationDetail_);
    auto *taskActions = new QHBoxLayout;
    auto *taskDetails = new QPushButton(tr("Task details"));
    cancelTask_ = new QPushButton(tr("Cancel pending task"));
    cancelTask_->setEnabled(false);
    taskActions->addWidget(taskDetails);
    taskActions->addWidget(cancelTask_);
    taskActions->addStretch();
    detailLayout->addLayout(taskActions);
    transcript_ = new QPlainTextEdit;
    transcript_->setObjectName("conversationTranscript");
    transcript_->setReadOnly(true);
    transcript_->setAccessibleName(tr("Conversation transcript"));
    transcript_->setPlaceholderText(tr("Saved messages appear here."));
    transcript_->setMaximumBlockCount(3500);
    detailLayout->addWidget(transcript_, 1);
    composer_ = new QPlainTextEdit;
    composer_->setObjectName("conversationComposer");
    composer_->setAccessibleName(tr("Message"));
    composer_->setPlaceholderText(tr("Describe a scoped task…"));
    composer_->setFixedHeight(90);
    detailLayout->addWidget(composer_);
    auto *commands = new QHBoxLayout;
    auto *limited = label(tr("Local intake · native Agent execution unavailable"), "meta");
    commands->addWidget(limited, 1);
    send_ = new QPushButton(tr("Send"));
    send_->setObjectName("sendMessage");
    send_->setProperty("primary", true);
    send_->setEnabled(false);
    commands->addWidget(send_);
    detailLayout->addLayout(commands);
    split->addWidget(detail);
    split->setStretchFactor(0, 1);
    split->setStretchFactor(1, 3);
    split->setSizes({220, 560});
    layout->addWidget(split, 1);

    auto *searchDelay = new QTimer(page);
    searchDelay->setSingleShot(true);
    searchDelay->setInterval(250);
    connect(conversationSearch_, &QLineEdit::textChanged, searchDelay, qOverload<>(&QTimer::start));
    connect(searchDelay, &QTimer::timeout, this, [this] { refreshConversations(); });
    connect(conversationMore_, &QPushButton::clicked, this, [this] { refreshConversations(true); });
    connect(conversationList_, &QListWidget::currentItemChanged, this, [this](QListWidgetItem *item) {
        if (item)
            openConversation(item->data(Qt::UserRole).toString());
    });
    connect(conversationRename_, &QPushButton::clicked, this, [this] { editConversation(false); });
    connect(conversationArchive_, &QPushButton::clicked, this, [this] { editConversation(true); });
    connect(newChat, &QPushButton::clicked, this, &MainWindow::newConversation);
    connect(send_, &QPushButton::clicked, this, &MainWindow::sendMessage);
    connect(taskDetails, &QPushButton::clicked, this, &MainWindow::conversationTaskDetails);
    connect(cancelTask_, &QPushButton::clicked, this, &MainWindow::cancelConversationTask);
    return page;
}

void MainWindow::refreshConversations(bool append)
{
    if (!online_)
        return;
    QJsonObject params{{"limit", 30}, {"query", conversationSearch_->text().trimmed()}};
    if (append && !conversationCursor_.isNull() && !conversationCursor_.isUndefined())
        params.insert("before", conversationCursor_);
    const auto request = ++conversationRequest_;
    conversationMore_->setEnabled(false);
    call("conversation.list", params, [this, request, append](const QJsonObject &data, const QString &error) {
        if (request != conversationRequest_)
            return;
        conversationMore_->setEnabled(true);
        if (!error.isEmpty()) {
            showStatus(error, true);
            return;
        }
        if (!append)
            conversationList_->clear();
        for (const auto &value : data.value("items").toArray()) {
            const auto row = value.toObject();
            auto *item = new QListWidgetItem(row.value("title").toString(), conversationList_);
            item->setToolTip((row.value("title").toString() + QStringLiteral("\n") + row.value("state").toString()).toHtmlEscaped());
            item->setData(Qt::UserRole, row.value("id").toString());
            if (row.value("id").toString() == currentConversation_)
                conversationList_->setCurrentItem(item);
        }
        conversationCursor_ = data.value("next_before");
        conversationMore_->setVisible(!conversationCursor_.isNull() && !conversationCursor_.isUndefined());
    });
}

void MainWindow::newConversation()
{
    if (!online_) {
        showStatus(tr("Connect the runtime to start a conversation."), true);
        return;
    }
    QDialog dialog(this);
    dialog.setWindowTitle(tr("New Chat"));
    dialog.resize(560, 360);
    auto *layout = new QVBoxLayout(&dialog);
    auto *fields = new QFormLayout;
    auto *title = new QLineEdit(tr("New conversation"));
    title->selectAll();
    title->setMaxLength(120);
    auto *profile = new QComboBox;
    profile->addItem(tr("Local handoff"), QStringLiteral("handoff"));
    fields->addRow(tr("Title"), title);
    fields->addRow(tr("Agent profile"), profile);
    layout->addLayout(fields);
    layout->addWidget(label(tr("Select one or more verified repository bindings."), "lead"));
    auto *bindings = new QListWidget;
    bindings->setSelectionMode(QAbstractItemView::MultiSelection);
    bindings->setAccessibleName(tr("Conversation repository bindings"));
    const auto selectedBinding = repositorySelection().value("binding_id").toString();
    for (const auto &value : repositories_) {
        const auto item = value.toObject();
        if (item.value("status").toString() == "ARCHIVED" || item.value("binding_id").toString().isEmpty())
            continue;
        auto *row = new QListWidgetItem(item.value("display_name").toString() + " · " + item.value("status").toString(), bindings);
        row->setToolTip(item.value("root").toString().toHtmlEscaped());
        row->setData(Qt::UserRole, item.value("binding_id").toString());
        if (row->data(Qt::UserRole).toString() == selectedBinding)
            row->setSelected(true);
    }
    layout->addWidget(bindings, 1);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
    auto *addRepo = buttons->addButton(tr("Add repository"), QDialogButtonBox::ActionRole);
    auto *ok = buttons->button(QDialogButtonBox::Ok);
    const auto enable = [ok, bindings, title] {
        ok->setEnabled(!bindings->selectedItems().isEmpty() && !title->text().trimmed().isEmpty());
    };
    enable();
    connect(bindings, &QListWidget::itemSelectionChanged, &dialog, enable);
    connect(title, &QLineEdit::textChanged, &dialog, enable);
    connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    connect(addRepo, &QPushButton::clicked, &dialog, [this, addRepo, &dialog] {
        QMenu menu(&dialog);
        auto *folder = menu.addAction(tr("Select folder"));
        auto *file = menu.addAction(tr("Open from file"));
        const auto choice = menu.exec(addRepo->mapToGlobal(QPoint(0, addRepo->height())));
        if (choice == folder || choice == file) {
            dialog.reject();
            QTimer::singleShot(0, this, [this, fromFile = choice == file] {
                showPage(repositoriesPage_);
                chooseRepository(fromFile);
            });
        }
    });
    layout->addWidget(buttons);
    if (dialog.exec() != QDialog::Accepted)
        return;
    QJsonArray ids;
    for (const auto *row : bindings->selectedItems())
        ids.append(row->data(Qt::UserRole).toString());
    call("conversation.create", {{"title", title->text().trimmed()}, {"bindings", ids},
                                  {"profile", profile->currentData().toString()}, {"key", requestKey()}},
         [this](const QJsonObject &result, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        const auto id = result.value("id").toString(result.value("conversation").toObject().value("id").toString());
        if (id.isEmpty()) {
            showError(tr("The runtime did not return the saved conversation."));
            return;
        }
        showPage(conversationsPage_);
        openConversation(id);
        refreshConversations();
    });
}

void MainWindow::openConversation(const QString &id)
{
    if (!online_ || id.isEmpty())
        return;
    if (id != currentConversation_) {
        currentConversation_ = id;
        eventCursor_ = 0;
        retryContent_.clear();
        retryMessageKey_.clear();
        composer_->clear();
        transcript_->clear();
    }
    const auto request = ++detailRequest_;
    call("conversation.get", {{"id", id}}, [this, id, request](const QJsonObject &result, const QString &error) {
        if (request != detailRequest_ || currentConversation_ != id)
            return;
        if (!error.isEmpty()) {
            showStatus(error, true);
            return;
        }
        conversation_ = result.value("conversation").toObject();
        conversationTasks_ = result.value("tasks").toArray();
        conversationBindings_ = result.value("bindings").toArray();
        conversationTitle_->setText(conversation_.value("title").toString(tr("Conversation")));
        const bool active = conversation_.value("state").toString() != "ARCHIVED";
        conversationRename_->setEnabled(active);
        conversationArchive_->setEnabled(active);
        send_->setEnabled(active);
        const auto latestTask = conversationTasks_.isEmpty() ? QJsonObject{} : conversationTasks_.last().toObject();
        const auto latestState = latestTask.value("state").toString();
        cancelTask_->setEnabled(active && (latestState == "PROPOSED" || latestState == "WAITING_APPROVAL"));
        QStringList messages;
        for (const auto &value : result.value("messages").toArray()) {
            const auto row = value.toObject();
            auto header = row.value("role").toString();
            if (row.value("status").isString())
                header += QStringLiteral(" · ") + row.value("status").toString();
            const auto task = row.value("task_id").toString();
            if (!task.isEmpty())
                header += QStringLiteral(" · ") + task;
            messages.append(header + "\n" + row.value("content").toString());
        }
        transcript_->setPlainText(messages.join(QStringLiteral("\n\n")));
        transcript_->moveCursor(QTextCursor::End);
        auto summary = tr("Profile %1 · %2 · %3")
            .arg(conversation_.value("profile").toString(), conversation_.value("state").toString(), id);
        if (!latestState.isEmpty())
            summary += QStringLiteral("\n") + tr("Latest task: %1").arg(latestState);
        conversationDetail_->setText(summary);
    });
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
    layout->addWidget(buttons);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    dialog.exec();
}

void MainWindow::cancelConversationTask()
{
    if (!online_ || conversationTasks_.isEmpty())
        return;
    const auto id = currentConversation_;
    const auto task = conversationTasks_.last().toObject().value("id");
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

void MainWindow::editConversation(bool archive)
{
    if (!online_ || conversation_.isEmpty())
        return;
    QJsonObject params{{"id", conversation_.value("id")}, {"revision", conversation_.value("revision")}};
    if (archive) {
        const auto answer = QMessageBox::question(this, tr("Archive conversation"),
            tr("Archive this conversation and keep its saved history?"),
            QMessageBox::Yes | QMessageBox::Cancel, QMessageBox::Cancel);
        if (answer != QMessageBox::Yes)
            return;
    } else {
        bool accepted = false;
        const auto title = QInputDialog::getText(this, tr("Rename conversation"), tr("Title"), QLineEdit::Normal,
                                                conversation_.value("title").toString(), &accepted).trimmed();
        if (!accepted || title.isEmpty())
            return;
        params.insert("title", title);
    }
    call(archive ? "conversation.archive" : "conversation.rename", params,
         [this](const QJsonObject &, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        openConversation(currentConversation_);
        refreshConversations();
    });
}

void MainWindow::sendMessage()
{
    if (!online_ || currentConversation_.isEmpty())
        return;
    const auto content = composer_->toPlainText().trimmed();
    if (content.isEmpty() || content.toUtf8().size() > 8192) {
        showStatus(tr("Enter a message of up to 8 KiB."), true);
        return;
    }
    if (content != retryContent_ || retryMessageKey_.isEmpty()) {
        retryContent_ = content;
        retryMessageKey_ = requestKey();
    }
    const auto id = currentConversation_;
    send_->setEnabled(false);
    call("conversation.send", {{"id", id}, {"content", content}, {"key", retryMessageKey_}},
         [this, id, content](const QJsonObject &result, const QString &error) {
        if (id != currentConversation_)
            return;
        send_->setEnabled(true);
        if (!error.isEmpty()) {
            showStatus(error, true);
            return;
        }
        retryContent_.clear();
        retryMessageKey_.clear();
        if (composer_->toPlainText().trimmed() == content)
            composer_->clear();
        const auto task = result.value("task").toObject();
        const auto state = task.value("status").toString(task.value("state").toString());
        showStatus(state.isEmpty() ? tr("Message saved.") : tr("Message saved · %1").arg(state));
        openConversation(id);
        refreshConversations();
    });
}

void MainWindow::refreshConversationEvents()
{
    if (!online_ || currentConversation_.isEmpty() || pages_->currentWidget() != conversationsPage_)
        return;
    const auto id = currentConversation_;
    call("conversation.events", {{"id", id}, {"after", qint64(eventCursor_)}, {"limit", 30}},
         [this, id](const QJsonObject &result, const QString &error) {
        if (!error.isEmpty() || id != currentConversation_)
            return;
        const auto next = quint64(qMax<qint64>(0, result.value("next_after").toInteger()));
        if (next > eventCursor_) {
            eventCursor_ = next;
            openConversation(id);
        }
    });
}
