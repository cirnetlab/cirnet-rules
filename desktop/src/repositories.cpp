#include "main_window.h"

#include <QCheckBox>
#include <QComboBox>
#include <QDateTime>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFileDialog>
#include <QFormLayout>
#include <QFontMetrics>
#include <QHeaderView>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QLabel>
#include <QLineEdit>
#include <QLocale>
#include <QMenu>
#include <QMessageBox>
#include <QPushButton>
#include <QSet>
#include <QStandardItemModel>
#include <QTableView>
#include <QTimeZone>
#include <QVBoxLayout>

namespace {
constexpr int repositoryRole = Qt::UserRole + 1;

QLabel *plainLabel(const QString &text, const char *name = nullptr)
{
    auto *label = new QLabel(text);
    label->setTextFormat(Qt::PlainText);
    if (name)
        label->setObjectName(QString::fromLatin1(name));
    return label;
}

QStandardItem *cell(const QString &text)
{
    auto *item = new QStandardItem(text);
    item->setEditable(false);
    item->setToolTip(text.toHtmlEscaped());
    return item;
}
}

QWidget *MainWindow::makeRepositories()
{
    auto *page = new QWidget;
    page->setObjectName("repositoriesPage");
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(36, 31, 36, 24);
    layout->setSpacing(12);
    layout->addWidget(plainLabel(tr("Repositories"), "heading"));
    auto *intro = plainLabel(
        tr("Choose a Git folder or a file inside it. Review the detected worktree before saving a binding."), "lead");
    intro->setWordWrap(true);
    layout->addWidget(intro);
    auto *add = new QHBoxLayout;
    auto *folder = new QPushButton(tr("Select folder"));
    folder->setObjectName("selectRepositoryFolder");
    folder->setProperty("primary", true);
    auto *file = new QPushButton(tr("Open from file"));
    file->setObjectName("selectRepositoryFile");
    auto *refresh = new QPushButton(tr("Refresh"));
    auto *archived = new QCheckBox(tr("Show archived"));
    add->addWidget(folder);
    add->addWidget(file);
    add->addWidget(refresh);
    add->addStretch();
    add->addWidget(archived);
    layout->addLayout(add);

    repositoryTable_ = new QTableView;
    repositoryTable_->setObjectName("repositoriesTable");
    repositoryTable_->setAccessibleName(tr("Repository bindings"));
    repositoryTable_->setSelectionBehavior(QAbstractItemView::SelectRows);
    repositoryTable_->setSelectionMode(QAbstractItemView::SingleSelection);
    repositoryTable_->setEditTriggers(QAbstractItemView::NoEditTriggers);
    repositoryTable_->setAlternatingRowColors(true);
    repositoryTable_->setWordWrap(false);
    repositoryTable_->setTextElideMode(Qt::ElideMiddle);
    repositoryTable_->verticalHeader()->setVisible(false);
    repositoryModel_ = new QStandardItemModel(0, 5, repositoryTable_);
    repositoryModel_->setHorizontalHeaderLabels({
        tr("Repository"), tr("Local Git root"), tr("Worktree / branch"), tr("State"), tr("Last selected")
    });
    repositoryTable_->setModel(repositoryModel_);
    repositoryTable_->horizontalHeader()->setSectionResizeMode(0, QHeaderView::ResizeToContents);
    repositoryTable_->horizontalHeader()->setSectionResizeMode(1, QHeaderView::Stretch);
    repositoryTable_->horizontalHeader()->setSectionResizeMode(2, QHeaderView::ResizeToContents);
    repositoryTable_->horizontalHeader()->setSectionResizeMode(3, QHeaderView::Fixed);
    repositoryTable_->setColumnWidth(3, qMax(108, QFontMetrics(repositoryTable_->font())
                                                      .horizontalAdvance(QStringLiteral("REBIND_REQUIRED")) + 28));
    repositoryTable_->horizontalHeader()->setSectionResizeMode(4, QHeaderView::ResizeToContents);
    layout->addWidget(repositoryTable_, 1);

    auto *actions = new QHBoxLayout;
    repoSelect_ = new QPushButton(tr("Use binding"));
    repoRelink_ = new QPushButton(tr("Relink"));
    repoRename_ = new QPushButton(tr("Rename"));
    repoArchive_ = new QPushButton(tr("Archive"));
    repoMore_ = new QPushButton(tr("Load more"));
    for (auto *button : {repoSelect_, repoRelink_, repoRename_, repoArchive_}) {
        button->setEnabled(false);
        actions->addWidget(button);
    }
    actions->addStretch();
    actions->addWidget(repoMore_);
    repoMore_->hide();
    layout->addLayout(actions);
    auto *limit = plainLabel(
        tr("Registration makes a repository available for selection. Task scope and write approval are separate."), "meta");
    limit->setWordWrap(true);
    layout->addWidget(limit);

    connect(folder, &QPushButton::clicked, this, [this] { chooseRepository(false); });
    connect(file, &QPushButton::clicked, this, [this] { chooseRepository(true); });
    connect(refresh, &QPushButton::clicked, this, &MainWindow::refreshRepositories);
    connect(archived, &QCheckBox::toggled, this, [this](bool checked) {
        includeArchived_ = checked;
        refreshRepositories();
    });
    connect(repoMore_, &QPushButton::clicked, this, &MainWindow::appendRepositories);
    connect(repoSelect_, &QPushButton::clicked, this, &MainWindow::selectRepository);
    connect(repoRename_, &QPushButton::clicked, this, &MainWindow::renameRepository);
    connect(repoArchive_, &QPushButton::clicked, this, &MainWindow::archiveRepository);
    connect(repoRelink_, &QPushButton::clicked, this, [this] {
        const auto repository = repositorySelection();
        if (repository.isEmpty())
            return;
        QMenu menu(this);
        auto *folderAction = menu.addAction(tr("Select folder"));
        auto *fileAction = menu.addAction(tr("Open from file"));
        const auto selected = menu.exec(repoRelink_->mapToGlobal(QPoint(0, repoRelink_->height())));
        if (selected == folderAction || selected == fileAction)
            chooseRepository(selected == fileAction, repository);
    });
    connect(repositoryTable_->selectionModel(), &QItemSelectionModel::selectionChanged,
            this, &MainWindow::updateRepositoryActions);
    return page;
}

void MainWindow::refreshRepositories()
{
    if (!online_) {
        showStatus(tr("Connect the runtime to manage repositories."), true);
        return;
    }
    repositories_ = {};
    repoCursor_ = QJsonValue();
    repositoryModel_->removeRows(0, repositoryModel_->rowCount());
    appendRepositories();
}

void MainWindow::appendRepositories()
{
    if (!online_)
        return;
    QJsonObject args{{"include_archived", includeArchived_}, {"limit", 40}};
    if (!repoCursor_.isNull() && !repoCursor_.isUndefined())
        args.insert("after", repoCursor_);
    const auto request = ++repoRequest_;
    repoMore_->setEnabled(false);
    call("repo.list", args, [this, request](const QJsonObject &data, const QString &error) {
        if (request != repoRequest_)
            return;
        repoMore_->setEnabled(true);
        if (!error.isEmpty()) {
            showStatus(error, true);
            return;
        }
        for (const auto &value : data.value("items").toArray()) {
            const auto item = value.toObject();
            repositories_.append(item);
            auto *display = cell(item.value("display_name").toString());
            display->setData(item, repositoryRole);
            auto branch = item.value("branch").toString(tr("Detached"));
            if (item.value("linked_worktree").toBool())
                branch = tr("Linked · %1").arg(branch);
            const auto seconds = item.value("selected_at").toInteger();
            const auto lastSelected = seconds > 0
                ? QLocale().toString(QDateTime::fromSecsSinceEpoch(seconds, QTimeZone::systemTimeZone()), QLocale::ShortFormat)
                : QStringLiteral("—");
            repositoryModel_->appendRow({display, cell(item.value("root").toString()),
                                          cell(branch), cell(item.value("status").toString()), cell(lastSelected)});
        }
        repoCursor_ = data.value("next");
        repoMore_->setVisible(!repoCursor_.isNull() && !repoCursor_.isUndefined());
        if (automationRepoFilter_) {
            const auto old = automationRepoFilter_->currentData();
            automationRepoFilter_->blockSignals(true);
            automationRepoFilter_->clear();
            automationRepoFilter_->addItem(tr("All repositories"), QString());
            for (const auto &value : repositories_) {
                const auto item = value.toObject();
                const auto binding = item.value("binding_id").toString();
                if (!binding.isEmpty())
                    automationRepoFilter_->addItem(item.value("display_name").toString(), binding);
            }
            const auto index = automationRepoFilter_->findData(old);
            automationRepoFilter_->setCurrentIndex(qMax(0, index));
            automationRepoFilter_->blockSignals(false);
        }
        showStatus(repositoryModel_->rowCount() == 0
                       ? tr("No repositories registered. Select a folder or file to begin.")
                       : tr("Repository bindings loaded."));
    });
}

QJsonObject MainWindow::repositorySelection() const
{
    const auto rows = repositoryTable_->selectionModel()->selectedRows();
    if (rows.isEmpty())
        return {};
    return rows.front().data(repositoryRole).toJsonObject();
}

void MainWindow::updateRepositoryActions()
{
    const auto repository = repositorySelection();
    const auto archived = repository.value("status").toString() == QStringLiteral("ARCHIVED");
    const bool allowed = online_ && !repository.isEmpty() && !archived;
    for (auto *button : {repoSelect_, repoRelink_, repoRename_, repoArchive_})
        button->setEnabled(allowed);
}

void MainWindow::chooseRepository(bool chooseFile, const QJsonObject &existing)
{
    if (!online_) {
        showStatus(tr("Connect the runtime to manage repositories."), true);
        return;
    }
    const auto path = chooseFile
        ? QFileDialog::getOpenFileName(this, tr("Open a file inside a repository"))
        : QFileDialog::getExistingDirectory(this, tr("Select a repository folder"));
    if (!path.isEmpty())
        previewRepository(path, existing);
}

void MainWindow::previewRepository(const QString &path, const QJsonObject &existing)
{
    showStatus(tr("Checking the selected Git worktree…"));
    call("repo.preview", {{"path", path}}, [this, path, existing](const QJsonObject &preview, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        const auto token = preview.value("token").toString();
        const auto root = preview.value("root").toString();
        if (root.isEmpty() || token.isEmpty()) {
            showError(tr("The runtime did not return a verified worktree."));
            return;
        }
        QDialog dialog(this);
        dialog.setWindowTitle(existing.isEmpty() ? tr("Confirm repository") : tr("Confirm relink"));
        dialog.resize(590, 290);
        auto *form = new QFormLayout(&dialog);
        auto *rootLabel = plainLabel(root);
        rootLabel->setWordWrap(true);
        rootLabel->setTextInteractionFlags(Qt::TextSelectableByMouse);
        form->addRow(tr("Git root"), rootLabel);
        form->addRow(tr("Worktree"), plainLabel(preview.value("linked_worktree").toBool()
                                                   ? tr("Linked worktree") : tr("Main checkout")));
        form->addRow(tr("Branch"), plainLabel(preview.value("branch").toString(tr("Detached"))));
        form->addRow(tr("Verified commit"), plainLabel(preview.value("commit").toString().left(12)));
        form->addRow(tr("Access"), plainLabel(preview.value("status").toString()));
        const auto hint = preview.value("file_hint").toString();
        if (!hint.isEmpty()) {
            auto *hintLabel = plainLabel(hint);
            hintLabel->setWordWrap(true);
            form->addRow(tr("File hint"), hintLabel);
        }
        auto *name = new QLineEdit(preview.value("suggested_name").toString(), &dialog);
        name->setMaxLength(100);
        auto *attach = new QComboBox(&dialog);
        if (existing.isEmpty()) {
            form->addRow(tr("Display name"), name);
            attach->addItem(tr("New logical repository"), QString());
            QSet<QString> listed;
            for (const auto &value : repositories_) {
                const auto row = value.toObject();
                const auto repoId = row.value("repo_id").toString();
                if (!repoId.isEmpty() && !listed.contains(repoId) &&
                    row.value("status").toString() != QStringLiteral("ARCHIVED")) {
                    listed.insert(repoId);
                    attach->addItem(row.value("display_name").toString(), repoId);
                }
            }
            form->addRow(tr("Register with"), attach);
            connect(attach, &QComboBox::currentIndexChanged, name,
                    [name, attach] { name->setEnabled(attach->currentData().toString().isEmpty()); });
        } else {
            name->hide();
            attach->hide();
            auto *warning = plainLabel(tr("Relinking replaces the saved worktree binding. Pending runs may need a new approval."));
            warning->setWordWrap(true);
            form->addRow(warning);
        }
        auto *buttons = new QDialogButtonBox(QDialogButtonBox::Save | QDialogButtonBox::Cancel);
        form->addRow(buttons);
        connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
        connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        if (dialog.exec() != QDialog::Accepted)
            return;

        QJsonObject params{{"path", path}, {"token", token}};
        QString method = QStringLiteral("repo.add");
        if (existing.isEmpty()) {
            const auto repo = attach->currentData().toString();
            if (repo.isEmpty())
                params.insert("display_name", name->text().trimmed());
            else
                params.insert("repo_id", repo);
        } else {
            method = QStringLiteral("repo.relink");
            params.insert("binding_id", existing.value("binding_id"));
            params.insert("revision", existing.value("revision"));
        }
        call(method, params, [this](const QJsonObject &, const QString &saveError) {
            if (!saveError.isEmpty()) {
                showError(saveError);
                return;
            }
            showStatus(tr("Repository binding saved."));
            refreshRepositories();
        });
    });
}

void MainWindow::renameRepository()
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty())
        return;
    bool confirmed = false;
    const auto name = QInputDialog::getText(this, tr("Rename repository"), tr("Display name"),
                                           QLineEdit::Normal, item.value("display_name").toString(), &confirmed);
    if (confirmed && !name.trimmed().isEmpty()) {
        call("repo.rename", {{"repo_id", item.value("repo_id")}, {"revision", item.value("repo_revision")},
                              {"display_name", name.trimmed()}},
             [this](const QJsonObject &, const QString &error) {
            if (!error.isEmpty())
                showError(error);
            else
                refreshRepositories();
        });
    }
}

void MainWindow::archiveRepository()
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty())
        return;
    QMessageBox dialog(QMessageBox::Question, tr("Archive repository"),
        tr("Archive %1? The source files and existing history stay on disk. Future dispatches using this repository are blocked.")
            .arg(item.value("display_name").toString()),
        QMessageBox::Yes | QMessageBox::Cancel, this);
    dialog.setTextFormat(Qt::PlainText);
    dialog.setDefaultButton(QMessageBox::Cancel);
    if (dialog.exec() != QMessageBox::Yes)
        return;
    call("repo.archive", {{"repo_id", item.value("repo_id")}, {"revision", item.value("repo_revision")}},
         [this](const QJsonObject &, const QString &error) {
        if (!error.isEmpty())
            showError(error);
        else
            refreshRepositories();
    });
}

void MainWindow::selectRepository()
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty())
        return;
    call("repo.select", {{"binding_id", item.value("binding_id")}, {"revision", item.value("revision")}},
         [this, item](const QJsonObject &value, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        showStatus(tr("%1 · %2").arg(item.value("display_name").toString(), value.value("status").toString()));
        if (value.value("status").toString() == "ACTIVE")
            newConversation();
        else
            refreshRepositories();
    });
}
