#include "main_window.h"
#include "governance_panel.h"
#include "live_text.h"
#include "full_row_table.h"

#include <QAction>
#include <QApplication>
#include <QCheckBox>
#include <QClipboard>
#include <QComboBox>
#include <QDateTime>
#include <QDesktopServices>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFileDialog>
#include <QFormLayout>
#include <QHeaderView>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QItemSelectionModel>
#include <QLabel>
#include <QLineEdit>
#include <QLocale>
#include <QMenu>
#include <QMessageBox>
#include <QPointer>
#include <QPushButton>
#include <QScopedValueRollback>
#include <QScrollBar>
#include <QSet>
#include <QSignalBlocker>
#include <QSortFilterProxyModel>
#include <QStandardItemModel>
#include <QStyle>
#include <QTimeZone>
#include <QTimer>
#include <QToolButton>
#include <QUrl>
#include <QVBoxLayout>

#include <algorithm>
#include <utility>

namespace {
constexpr int repositoryRole = Qt::UserRole + 1;
constexpr int repositorySortRole = Qt::UserRole + 2;
constexpr int pageSize = 40;
constexpr int columnCount = 5;
constexpr int defaultWidths[columnCount] = {150, 290, 145, 125, 145};

bool hasCursor(const QJsonValue &cursor)
{
    return cursor.isString() && !cursor.toString().isEmpty();
}

QString bindingId(const QJsonObject &item)
{
    return item.value("binding_id").toString();
}

QString tooltip(const QString &text)
{
    return QStringLiteral("<qt>%1</qt>").arg(text.toHtmlEscaped());
}

QLabel *plainLabel(const QString &text, const char *name = nullptr)
{
    auto *label = new QLabel(text);
    label->setTextFormat(Qt::PlainText);
    if (name)
        label->setObjectName(QString::fromLatin1(name));
    return label;
}

#ifdef Q_OS_WIN
const auto extendedPrefix = QStringLiteral(R"(\\?\)");

QString ordinaryDrivePath(const QString &path)
{
    if (!path.startsWith(extendedPrefix) || path.size() < 7
        || !((path[4] >= u'A' && path[4] <= u'Z') || (path[4] >= u'a' && path[4] <= u'z'))
        || path[5] != u':' || path[6] != u'\\')
        return {};

    const auto relative = QStringView(path).sliced(7);
    for (const auto component : relative.split(u'\\', Qt::SkipEmptyParts)) {
        if (component.endsWith(u'.') || component.endsWith(u' ') || component.contains(u'/'))
            return {};
    }
    // The runtime verifies that canonical drive paths exclude device names,
    // unsupported components and reparse points before any operation.
    return path.mid(4);
}
#endif

// Simplify a display only if the ordinary drive path has the same components.
// Keep the runtime value for binding operations, details and clipboard.
QString displayPath(const QString &path)
{
#ifdef Q_OS_WIN
    const auto ordinary = ordinaryDrivePath(path);
    if (!ordinary.isEmpty())
        return ordinary;
#endif
    return path;
}

QUrl folderUrl(const QString &verifiedRoot)
{
#ifdef Q_OS_WIN
    // A verified local disk root can be encoded as a drive-letter file URI.
    // Qt would otherwise interpret the Win32 extended-length prefix as a host.
    if (verifiedRoot.startsWith(extendedPrefix)) {
        const auto ordinary = ordinaryDrivePath(verifiedRoot);
        return ordinary.isEmpty() ? QUrl() : QUrl::fromLocalFile(ordinary);
    }
#endif
    return QUrl::fromLocalFile(verifiedRoot);
}

QString formattedTime(qint64 seconds)
{
    if (seconds <= 0)
        return QStringLiteral("—");
    return QLocale().toString(QDateTime::fromSecsSinceEpoch(seconds, QTimeZone::systemTimeZone()),
                             QLocale::ShortFormat);
}

class RepositoryFilter final : public QSortFilterProxyModel {
public:
    explicit RepositoryFilter(QObject *parent) : QSortFilterProxyModel(parent)
    {
        setSortCaseSensitivity(Qt::CaseInsensitive);
        setSortRole(repositorySortRole);
    }

    void setFilters(const QString &search, const QString &status, bool archived)
    {
        search_ = search.trimmed();
        status_ = status;
        archived_ = archived;
        invalidateFilter();
    }

protected:
    bool filterAcceptsRow(int row, const QModelIndex &parent) const override
    {
        const auto item = sourceModel()->index(row, 0, parent).data(repositoryRole).toJsonObject();
        const auto status = item.value("status").toString();
        if (!archived_ && status == QStringLiteral("ARCHIVED"))
            return false;
        if (status_ != QStringLiteral("all")) {
            const auto selected = status_ == QStringLiteral("rebind")
                ? QStringLiteral("REBIND_REQUIRED") : status_.toUpper();
            if (status != selected)
                return false;
        }
        if (search_.isEmpty())
            return true;
        for (const auto *field : {"display_name", "root", "branch", "file_hint"}) {
            if (item.value(QLatin1String(field)).toString().contains(search_, Qt::CaseInsensitive))
                return true;
        }
        return false;
    }

private:
    QString search_;
    QString status_ = QStringLiteral("all");
    bool archived_ = false;
};

void updateCell(QStandardItem *item, const QString &text, const QString &detail, const QVariant &key)
{
    item->setEditable(false);
    item->setText(text);
    item->setToolTip(tooltip(detail));
    item->setData(key, repositorySortRole);
}
}

QWidget *MainWindow::makeRepositories()
{
    auto *page = new QWidget;
    page->setObjectName("repositoriesPage");
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(10);
    layout->addWidget(live::text(plainLabel({}, "heading"), [] { return tr("Repositories"); }));

    auto *tools = new QHBoxLayout;
    auto *add = new QToolButton;
    add->setObjectName("addRepository");
    live::text(add, [] { return tr("Add repository"); });
    add->setPopupMode(QToolButton::InstantPopup);
    auto *pickMenu = new QMenu(add);
    auto *folder = live::text(pickMenu->addAction(QString{}), [] { return tr("Select folder"); });
    folder->setObjectName("selectRepositoryFolder");
    auto *file = live::text(pickMenu->addAction(QString{}), [] { return tr("Open from file"); });
    file->setObjectName("selectRepositoryFile");
    add->setMenu(pickMenu);
    auto *refresh = new QPushButton;
    refresh->setObjectName("repoRefresh");
    refresh->setIconSize(QSize(20, 20));
    const auto refreshIcon = refresh->style()->standardIcon(QStyle::SP_BrowserReload);
    refresh->setIcon(refreshIcon);
    if (refreshIcon.isNull())
        live::text(refresh, [] { return tr("Refresh"); });
    live::text(refresh, [] { return tr("Refresh"); }, &QWidget::setToolTip);
    live::text(refresh, [] { return tr("Refresh"); }, &QWidget::setAccessibleName);
    repoArchived_ = live::text(new QCheckBox, [] { return tr("Show archived"); });
    repoArchived_->setObjectName("repoShowArchived");
    tools->addWidget(add);
    tools->addWidget(refresh);
    tools->addStretch(1);
    tools->addWidget(repoArchived_);
    layout->addLayout(tools);

    auto *filters = new QHBoxLayout;
    repoSearch_ = new QLineEdit;
    repoSearch_->setObjectName("repoSearch");
    repoSearch_->setProperty("searchField", true);
    live::placeholder(repoSearch_, [] { return tr("Search name, path or branch"); });
    live::text(repoSearch_, [] { return tr("Search repositories"); }, &QWidget::setAccessibleName);
    repoSearch_->setClearButtonEnabled(true);
    repoStateFilter_ = new QComboBox;
    repoStateFilter_->setObjectName("repoStateFilter");
    live::text(repoStateFilter_, [] { return tr("Filter repository status"); }, &QWidget::setAccessibleName);
    live::item(repoStateFilter_, [] { return tr("All statuses"); }, "all");
    live::item(repoStateFilter_, [] { return tr("Active bindings"); }, "active");
    live::item(repoStateFilter_, [] { return tr("Missing"); }, "missing");
    live::item(repoStateFilter_, [] { return tr("Inaccessible"); }, "inaccessible");
    live::item(repoStateFilter_, [] { return tr("Needs relink"); }, "rebind");
    live::item(repoStateFilter_, [] { return tr("Archived"); }, "archived");
    filters->addWidget(repoSearch_, 1);
    filters->addWidget(repoStateFilter_);
    layout->addLayout(filters);

    repositoryTable_ = new FullRowTable;
    repositoryTable_->setObjectName("repositoriesTable");
    live::text(repositoryTable_, [] { return tr("Repository bindings"); }, &QWidget::setAccessibleName);
    repositoryTable_->setWordWrap(false);
    repositoryTable_->setTextElideMode(Qt::ElideMiddle);
    repositoryTable_->setHorizontalScrollMode(QAbstractItemView::ScrollPerPixel);
    repositoryTable_->setVerticalScrollMode(QAbstractItemView::ScrollPerPixel);
    repositoryTable_->verticalHeader()->setVisible(false);
    repositoryModel_ = new QStandardItemModel(0, columnCount, repositoryTable_);
    live::bind(repositoryModel_, [this] {
        repositoryModel_->setHorizontalHeaderLabels({
            tr("Repository"), tr("Local path"), tr("Worktree / branch"), tr("Health"), tr("Last used")
        });
    });
    repositoryProxy_ = new RepositoryFilter(repositoryTable_);
    repositoryProxy_->setSourceModel(repositoryModel_);
    repositoryTable_->setModel(repositoryProxy_);
    auto *header = repositoryTable_->horizontalHeader();
    header->setSectionResizeMode(QHeaderView::Interactive);
    header->setSectionsMovable(true);
    header->setMinimumSectionSize(48);
    header->setMaximumSectionSize(4096);
    header->setStretchLastSection(false);
    header->setDefaultAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    for (int i = 0; i < columnCount; ++i)
        header->resizeSection(i, defaultWidths[i]);
    repositoryTable_->setSortingEnabled(true);
    repositoryTable_->sortByColumn(0, Qt::AscendingOrder);
    layout->addWidget(repositoryTable_, 1);

    auto *actions = new QHBoxLayout;
    repoSelect_ = live::text(new QPushButton, [] { return tr("Use binding"); });
    repoSelect_->setObjectName("repoUseBinding");
    auto *moreActions = new QToolButton;
    moreActions->setObjectName("repoActions");
    live::text(moreActions, [] { return tr("Actions"); });
    moreActions->setPopupMode(QToolButton::InstantPopup);
    auto *actionMenu = new QMenu(moreActions);
    auto *relink = live::text(actionMenu->addMenu(QString{}), [] { return tr("Relink"); }, &QMenu::setTitle);
    repoRelink_ = relink->menuAction();
    auto *relinkFolder = live::text(relink->addAction(QString{}), [] { return tr("Select folder"); });
    relinkFolder->setObjectName("repoRelinkFolder");
    auto *relinkFile = live::text(relink->addAction(QString{}), [] { return tr("Open from file"); });
    relinkFile->setObjectName("repoRelinkFile");
    repoRename_ = live::text(actionMenu->addAction(QString{}), [] { return tr("Rename"); });
    repoRename_->setObjectName("repoRename");
    repoArchive_ = live::text(actionMenu->addAction(QString{}), [] { return tr("Archive"); });
    repoArchive_->setObjectName("repoArchive");
    actionMenu->addSeparator();
    repoCheck_ = live::text(actionMenu->addAction(QString{}), [] { return tr("Check health"); });
    repoCheck_->setObjectName("repoCheckHealth");
    repoCopy_ = live::text(actionMenu->addAction(QString{}), [] { return tr("Copy path"); });
    repoCopy_->setObjectName("repoCopyPath");
    repoOpen_ = live::text(actionMenu->addAction(QString{}), [] { return tr("Open folder"); });
    repoOpen_->setObjectName("repoOpenFolder");
    actionMenu->addSeparator();
    auto *resetColumns = live::text(actionMenu->addAction(QString{}), [] { return tr("Reset columns"); });
    resetColumns->setObjectName("repoResetColumns");
    moreActions->setMenu(actionMenu);
    repoMore_ = live::text(new QPushButton, [] { return tr("Load more"); });
    repoMore_->setObjectName("repoLoadMore");
    actions->addWidget(repoSelect_);
    actions->addWidget(moreActions);
    actions->addStretch();
    actions->addWidget(repoMore_);
    repoMore_->hide();
    layout->addLayout(actions);

    repoSummary_ = plainLabel({}, "repoSummary");
    live::message(repoSummary_, [] { return tr("Repository catalog is unavailable."); });
    repoSummary_->setProperty("tone", "muted");
    repoSummary_->setWordWrap(true);
    layout->addWidget(repoSummary_);
    repoDetails_ = plainLabel(QString(), "repoDetails");
    repoDetails_->setWordWrap(true);
    repoDetails_->hide();
    layout->addWidget(repoDetails_);
    auto *limit = live::text(plainLabel({}, "meta"), [] { return tr("Repository registration makes a binding available. Task scope and write approval are separate."); });
    limit->setWordWrap(true);
    layout->addWidget(limit);

    connect(folder, &QAction::triggered, this, [this] { chooseRepository(false); });
    connect(file, &QAction::triggered, this, [this] { chooseRepository(true); });
    connect(refresh, &QPushButton::clicked, this, [this] {
        if (online_)
            refreshRepositories();
        else
            showStatus(tr("Connect the runtime to manage repositories."), true);
    });
    connect(repoSearch_, &QLineEdit::textChanged, this, &MainWindow::applyRepositoryFilter);
    connect(repoStateFilter_, &QComboBox::currentIndexChanged, this, [this] {
        if (repoRestoring_)
            return;
        if (repoStateFilter_->currentData().toString() == QStringLiteral("archived"))
            repoArchived_->setChecked(true);
        applyRepositoryFilter();
        schedulePreferencesSave();
    });
    connect(repoArchived_, &QCheckBox::toggled, this, [this](bool checked) {
        if (repoRestoring_)
            return;
        if (!checked && repoStateFilter_->currentData().toString() == QStringLiteral("archived")) {
            const QSignalBlocker blocker(repoStateFilter_);
            repoStateFilter_->setCurrentIndex(0);
        }
        applyRepositoryFilter();
        schedulePreferencesSave();
    });
    connect(repoMore_, &QPushButton::clicked, this, &MainWindow::appendRepositories);
    connect(repoSelect_, &QPushButton::clicked, this, &MainWindow::selectRepository);
    connect(repoRename_, &QAction::triggered, this, &MainWindow::renameRepository);
    connect(repoArchive_, &QAction::triggered, this, &MainWindow::archiveRepository);
    connect(relinkFolder, &QAction::triggered, this, [this] {
        const auto item = repositorySelection();
        if (!item.isEmpty())
            chooseRepository(false, item);
    });
    connect(relinkFile, &QAction::triggered, this, [this] {
        const auto item = repositorySelection();
        if (!item.isEmpty())
            chooseRepository(true, item);
    });
    connect(repoCheck_, &QAction::triggered, this, [this] { checkRepository(false); });
    connect(repoOpen_, &QAction::triggered, this, [this] { checkRepository(true); });
    connect(repoCopy_, &QAction::triggered, this, [this] {
        const auto path = repositorySelection().value("root").toString();
        if (!path.isEmpty()) {
            auto *clipboard = QApplication::clipboard();
            clipboard->setText(path);
            if (clipboard->text() == path)
                showStatus(tr("Canonical path copied."));
            else
                showStatus(tr("Could not copy the canonical path."), true);
        }
    });
    connect(resetColumns, &QAction::triggered, this, [this, header] {
        {
            const QScopedValueRollback restore(repoRestoring_, true);
            for (int i = 0; i < columnCount; ++i) {
                header->moveSection(header->visualIndex(i), i);
                header->resizeSection(i, defaultWidths[i]);
            }
            repositoryTable_->sortByColumn(0, Qt::AscendingOrder);
        }
        schedulePreferencesSave();
    });
    connect(header, &QHeaderView::sectionHandleDoubleClicked, repositoryTable_, &QTableView::resizeColumnToContents);
    const auto saveLayout = [this] {
        if (!repoRestoring_)
            schedulePreferencesSave();
    };
    connect(header, &QHeaderView::sectionResized, this, saveLayout);
    connect(header, &QHeaderView::sectionMoved, this, saveLayout);
    connect(header, &QHeaderView::sortIndicatorChanged, this, saveLayout);
    connect(repositoryTable_->verticalScrollBar(), &QScrollBar::actionTriggered, this, [this, saveLayout] {
        if (!repoRestoring_) {
            repositoryScroll_ = repositoryTable_->verticalScrollBar()->sliderPosition();
            saveLayout();
        }
    });
    connect(repositoryTable_->horizontalScrollBar(), &QScrollBar::actionTriggered, this, [this, saveLayout] {
        if (!repoRestoring_) {
            repositoryHScroll_ = repositoryTable_->horizontalScrollBar()->sliderPosition();
            saveLayout();
        }
    });
    connect(repositoryTable_->selectionModel(), &QItemSelectionModel::selectionChanged, this, [this, saveLayout] {
        if (!repoRestoring_) {
            repositorySelectedId_ = bindingId(repositorySelection());
            saveLayout();
        }
        updateRepositoryActions();
    });
    updateRepositoryActions();
    return page;
}

QJsonObject MainWindow::repositoryLayout() const
{
    if (!repositoryTable_)
        return {};
    auto *header = repositoryTable_->horizontalHeader();
    QJsonArray widths;
    QJsonArray order;
    for (int i = 0; i < columnCount; ++i) {
        widths.append(header->sectionSize(i));
        order.append(header->logicalIndex(i));
    }
    return {{"widths", widths}, {"order", order}, {"sort_column", header->sortIndicatorSection()},
            {"sort_descending", header->sortIndicatorOrder() == Qt::DescendingOrder},
            {"show_archived", repoArchived_->isChecked()}, {"status", repoStateFilter_->currentData().toString()},
            {"selected_id", repositorySelectedId_.isEmpty() ? QJsonValue() : QJsonValue(repositorySelectedId_)},
            {"scroll", repositoryScroll_}, {"horizontal_scroll", repositoryHScroll_}};
}

void MainWindow::restoreRepositoryLayout(const QJsonObject &layout)
{
    if (!repositoryTable_)
        return;
    const QScopedValueRollback restore(repoRestoring_, true);
    auto *header = repositoryTable_->horizontalHeader();
    const auto widths = layout.value("widths").toArray();
    const auto order = layout.value("order").toArray();
    for (int i = 0; i < columnCount; ++i) {
        const auto width = widths.size() == columnCount ? widths.at(i).toInt(defaultWidths[i]) : defaultWidths[i];
        header->resizeSection(i, qBound(48, width, 4096));
        const auto logical = order.size() == columnCount ? order.at(i).toInt(i) : i;
        if (logical >= 0 && logical < columnCount)
            header->moveSection(header->visualIndex(logical), i);
    }
    const auto column = layout.value("sort_column").toInt(0);
    repositoryTable_->sortByColumn(column, layout.value("sort_descending").toBool()
                                             ? Qt::DescendingOrder : Qt::AscendingOrder);
    const auto status = layout.value("status").toString();
    repoArchived_->setChecked(layout.value("show_archived").toBool() || status == QStringLiteral("archived"));
    repoStateFilter_->setCurrentIndex(qMax(0, repoStateFilter_->findData(status)));
    repositorySelectedId_ = layout.value("selected_id").toString();
    repositoryScroll_ = layout.value("scroll").toInt();
    repositoryHScroll_ = layout.value("horizontal_scroll").toInt();
    applyRepositoryFilter();
}

void MainWindow::refreshRepositories()
{
    if (!online_)
        return;
    ++repoRequest_;
    repoLoading_ = true;
    repoComplete_ = false;
    repoIncoming_ = {};
    repoIncomingCursor_ = {};
    repoRefreshRows_ = qMax(pageSize, int(repositories_.size()));
    fetchRepositoryPage();
}

void MainWindow::appendRepositories()
{
    if (!online_ || repoLoading_ || repoComplete_)
        return;
    ++repoRequest_;
    repoLoading_ = true;
    repoIncoming_ = repositories_;
    repoIncomingCursor_ = repoCursor_;
    repoRefreshRows_ = int(repositories_.size()) + pageSize;
    fetchRepositoryPage();
}

void MainWindow::ensureRepositoryCatalog(std::function<void(bool)> done)
{
    if (!online_ || repoComplete_) {
        QTimer::singleShot(0, this, [done = std::move(done), complete = online_ && repoComplete_] {
            done(complete);
        });
        return;
    }
    repoWaiters_.append(std::move(done));
    if (!repoLoading_)
        appendRepositories();
}

void MainWindow::fetchRepositoryPage()
{
    repoMore_->setEnabled(false);
    live::message(repoSummary_, [] { return tr("Loading repository bindings…"); });
    QJsonObject args{{"include_archived", true}, {"limit", pageSize}};
    if (hasCursor(repoIncomingCursor_))
        args.insert("after", repoIncomingCursor_);
    const auto request = repoRequest_;
    call("repo.list", args, [this, request](const QJsonObject &data, const QString &error) {
        if (request != repoRequest_)
            return;
        const auto next = data.value("next");
        const bool valid = error.isEmpty() && data.value("items").isArray()
            && (!hasCursor(next) || next != repoIncomingCursor_);
        if (!valid) {
            repoLoading_ = false;
            repoMore_->setEnabled(true);
            live::message(repoSummary_, [] { return tr("The repository catalog could not be refreshed. Previously loaded rows are retained."); });
            showStatus(error.isEmpty() ? tr("The runtime returned an invalid catalog page.") : error, true);
            auto waiters = std::exchange(repoWaiters_, {});
            QTimer::singleShot(0, this, [waiters = std::move(waiters)] {
                for (const auto &done : waiters)
                    done(false);
            });
            return;
        }
        for (const auto &value : data.value("items").toArray())
            repoIncoming_.append(value);
        repoIncomingCursor_ = next;
        const auto hasSelected = std::any_of(repoIncoming_.cbegin(), repoIncoming_.cend(), [this](const auto &row) {
            return bindingId(row.toObject()) == repositorySelectedId_;
        });
        if (hasCursor(next) && (repoIncoming_.size() < repoRefreshRows_
                                || !repoWaiters_.isEmpty()
                                || (!repositorySelectedId_.isEmpty() && !hasSelected))) {
            fetchRepositoryPage();
            return;
        }
        repositories_ = std::exchange(repoIncoming_, {});
        governance_->repositoryCatalogChanged();
        repoCursor_ = next;
        repoComplete_ = !hasCursor(next);
        repoLoading_ = false;
        repoMore_->setEnabled(true);
        renderRepositoryRows();
        renderAutomationCards();
        updateConversationSummary();
        auto waiters = std::exchange(repoWaiters_, {});
        QTimer::singleShot(0, this, [waiters = std::move(waiters), complete = repoComplete_] {
            for (const auto &done : waiters)
                done(complete);
        });
    });
}

void MainWindow::renderRepositoryRows()
{
    const QScopedValueRollback restore(repoRestoring_, true);
    QSet<QString> ids;
    for (const auto &value : repositories_)
        ids.insert(bindingId(value.toObject()));
    for (int row = repositoryModel_->rowCount() - 1; row >= 0; --row) {
        const auto id = bindingId(repositoryModel_->item(row, 0)->data(repositoryRole).toJsonObject());
        if (!ids.contains(id)) {
            repositoryModel_->removeRow(row);
            repositoryHealth_.remove(id);
        }
    }
    QHash<QString, int> rows;
    for (int row = 0; row < repositoryModel_->rowCount(); ++row)
        rows.insert(bindingId(repositoryModel_->item(row, 0)->data(repositoryRole).toJsonObject()), row);

    for (const auto &value : repositories_) {
        auto item = value.toObject();
        const auto id = bindingId(item);
        const auto health = repositoryHealth_.value(id);
        if (!health.isEmpty() && item.value("revision") == health.value("revision")
            && item.value("root") == health.value("root")
            && item.value("status").toString() == QStringLiteral("ACTIVE")) {
            for (const auto *field : {"status", "branch", "warning", "checked_at"})
                item.insert(QLatin1String(field), health.value(QLatin1String(field)));
        }
        int row = rows.value(id, -1);
        if (row >= 0 && repositoryModel_->item(row, 0)->data(repositoryRole).toJsonObject() == item)
            continue;
        if (row < 0) {
            row = repositoryModel_->rowCount();
            repositoryModel_->appendRow({new QStandardItem, new QStandardItem, new QStandardItem,
                                         new QStandardItem, new QStandardItem});
        }
        const auto name = item.value("display_name").toString();
        auto *display = repositoryModel_->item(row, 0);
        display->setData(item, repositoryRole);
        updateCell(display, name, name, name);
        const auto root = item.value("root").toString();
        updateCell(repositoryModel_->item(row, 1), displayPath(root), root, displayPath(root));
        auto branch = item.value("branch").toString(tr("Detached"));
        if (item.value("linked_worktree").toBool())
            branch = tr("Linked · %1").arg(branch);
        updateCell(repositoryModel_->item(row, 2), branch, branch, branch);
        const auto status = item.value("status").toString();
        QString statusLabel = uiStatusLabel(status);
        if (status == QStringLiteral("ACTIVE"))
            statusLabel = item.contains("checked_at") ? tr("Verified") : tr("Not checked");
        else if (status == QStringLiteral("MISSING"))
            statusLabel = tr("Missing");
        else if (status == QStringLiteral("INACCESSIBLE"))
            statusLabel = tr("Inaccessible");
        else if (status == QStringLiteral("REBIND_REQUIRED"))
            statusLabel = tr("Needs relink");
        else if (status == QStringLiteral("ARCHIVED"))
            statusLabel = tr("Archived");
        updateCell(repositoryModel_->item(row, 3), statusLabel,
                   item.value("warning").toString(statusLabel), statusLabel);
        const auto seconds = item.value("selected_at").toInteger();
        const auto lastUsed = formattedTime(seconds);
        updateCell(repositoryModel_->item(row, 4), lastUsed, lastUsed, seconds);
    }
    applyRepositoryFilter();
}

void MainWindow::applyRepositoryFilter()
{
    const QScopedValueRollback restore(repoRestoring_, true);
    auto *proxy = static_cast<RepositoryFilter *>(repositoryProxy_);
    proxy->setFilters(repoSearch_->text(), repoStateFilter_->currentData().toString(), repoArchived_->isChecked());
    auto *selection = repositoryTable_->selectionModel();
    bool found = false;
    for (int row = 0; row < repositoryProxy_->rowCount(); ++row) {
        const auto index = repositoryProxy_->index(row, 0);
        if (bindingId(index.data(repositoryRole).toJsonObject()) == repositorySelectedId_) {
            selection->setCurrentIndex(index, QItemSelectionModel::ClearAndSelect | QItemSelectionModel::Rows);
            found = true;
            break;
        }
    }
    if (!found)
        selection->clear();
    restoreRepositoryScroll();
    repoMore_->setVisible(!repoComplete_ && hasCursor(repoCursor_));
    if (!repoLoading_ && (repoComplete_ || !repositories_.isEmpty())) {
        live::message(repoSummary_, [visible = repositoryProxy_->rowCount(), loaded = repositories_.size(), complete = repoComplete_] {
            const auto shown = tr("%1 shown · %2 loaded").arg(visible).arg(loaded);
            return complete ? shown : tr("%1 · Search covers loaded rows.").arg(shown);
        });
    }
    updateRepositoryActions();
}

void MainWindow::restoreRepositoryScroll()
{
    const QScopedValueRollback restore(repoRestoring_, true);
    repositoryTable_->verticalScrollBar()->setValue(repositoryScroll_);
    repositoryTable_->horizontalScrollBar()->setValue(repositoryHScroll_);
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
    if (!repositoryTable_)
        return;
    const auto item = repositorySelection();
    const auto status = item.value("status").toString();
    const bool archived = status == QStringLiteral("ARCHIVED");
    const bool allowed = online_ && !item.isEmpty() && !archived && !repoActionPending_;
    const bool available = allowed && status == QStringLiteral("ACTIVE");
    repoSelect_->setEnabled(available);
    repoRelink_->setEnabled(allowed);
    repoRename_->setEnabled(allowed);
    repoArchive_->setEnabled(allowed);
    repoCheck_->setEnabled(allowed);
    repoOpen_->setEnabled(available);
    repoCopy_->setEnabled(!item.value("root").toString().isEmpty());
    repoDetails_->setVisible(!item.isEmpty());
    if (!item.isEmpty()) {
        const auto registration = archived ? tr("Archived") : tr("Registered");
        auto detail = tr("%1 · Last verified %2").arg(registration, formattedTime(item.value("verified_at").toInteger()));
        const auto checked = item.value("checked_at").toInteger();
        if (checked > 0)
            detail += tr(" · Health checked %1").arg(formattedTime(checked));
        const auto warning = item.value("warning").toString();
        if (!warning.isEmpty())
            detail += QStringLiteral("\n") + warning;
        repoDetails_->setText(detail);
    }
}

void MainWindow::checkRepository(bool openFolder)
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty() || repoActionPending_)
        return;
    repoActionPending_ = true;
    updateRepositoryActions();
    call("repo.health", {{"binding_id", item.value("binding_id")}, {"revision", item.value("revision")}},
         [this, openFolder, item](const QJsonObject &result, const QString &error) {
        repoActionPending_ = false;
        updateRepositoryActions();
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        auto health = result;
        health.insert("checked_at", QDateTime::currentSecsSinceEpoch());
        repositoryHealth_.insert(bindingId(item), health);
        renderRepositoryRows();
        const auto status = result.value("status").toString();
        if (status != QStringLiteral("ACTIVE")) {
            showStatus(result.value("warning").toString(tr("The worktree could not be verified. Review its health or relink it.")), true);
            return;
        }
        if (!openFolder) {
            showStatus(tr("Git worktree verified."));
            return;
        }
        const auto root = result.value("root").toString();
        const auto url = folderUrl(root);
        if (root.isEmpty() || root != item.value("root").toString() || !url.isLocalFile()
            || !QDesktopServices::openUrl(url))
            showError(tr("The system could not open this folder. Use Copy path to inspect it."));
    });
}

void MainWindow::chooseRepository(bool chooseFile, const QJsonObject &existing)
{
    if (!online_ || repoActionPending_) {
        if (!online_)
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
    repoActionPending_ = true;
    updateRepositoryActions();
    call("repo.preview", {{"path", path}}, [this, path, existing](const QJsonObject &preview, const QString &error) {
        if (!error.isEmpty()) {
            repoActionPending_ = false;
            updateRepositoryActions();
            showError(tr("Could not verify the selected path.\n%1").arg(error));
            return;
        }
        const auto token = preview.value("token").toString();
        const auto root = preview.value("root").toString();
        if (root.isEmpty() || token.isEmpty()) {
            repoActionPending_ = false;
            updateRepositoryActions();
            showError(tr("The runtime did not return a verified worktree."));
            return;
        }

        QTimer::singleShot(0, this, [this, path, existing, preview] {
            confirmRepository(path, existing, preview);
        });
    });
}

void MainWindow::confirmRepository(const QString &path, const QJsonObject &existing,
                                   const QJsonObject &preview)
{
    if (!online_ || !repositoriesPage_->isVisible()) {
        repoActionPending_ = false;
        updateRepositoryActions();
        return;
    }
    const auto token = preview.value("token").toString();
    const auto root = preview.value("root").toString();
    QDialog dialog(this);
    dialog.setObjectName("repositoryConfirmation");
    dialog.setWindowTitle(existing.isEmpty() ? tr("Confirm repository") : tr("Confirm relink"));
    dialog.resize(540, 300);
    auto *form = new QFormLayout(&dialog);
    form->setFieldGrowthPolicy(QFormLayout::AllNonFixedFieldsGrow);
    form->setRowWrapPolicy(QFormLayout::WrapLongRows);
    const auto addPath = [form](const QString &caption, const QString &canonical) {
        auto *field = new QLineEdit(displayPath(canonical));
        field->setReadOnly(true);
        field->setToolTip(tooltip(canonical));
        form->addRow(caption, field);
    };
    addPath(tr("Selected path"), path);
    addPath(tr("Git root"), root);
    form->addRow(tr("Worktree"), plainLabel(preview.value("linked_worktree").toBool()
                                               ? tr("Linked worktree") : tr("Main checkout")));
    form->addRow(tr("Branch"), plainLabel(preview.value("branch").toString(tr("Detached"))));
    const auto commit = preview.value("commit").toString();
    form->addRow(tr("Verified commit"), plainLabel(commit.isEmpty() ? tr("No commits yet") : commit.left(12)));
    const auto hint = preview.value("file_hint").toString();
    if (!hint.isEmpty())
        addPath(tr("File hint"), hint);

    QLineEdit *name = nullptr;
    QComboBox *attach = nullptr;
    if (existing.isEmpty()) {
        name = new QLineEdit(preview.value("suggested_name").toString());
        name->setMaxLength(100);
        form->addRow(tr("Display name"), name);
        attach = new QComboBox;
        attach->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
        attach->addItem(tr("New logical repository"), QString());
        form->addRow(tr("Register with"), attach);
        connect(attach, &QComboBox::currentIndexChanged, name,
                [name, attach] { name->setEnabled(attach->currentData().toString().isEmpty()); });
        const QPointer<QComboBox> liveAttach(attach);
        ensureRepositoryCatalog([this, liveAttach](bool complete) {
            if (!liveAttach || !complete)
                return;
            QSet<QString> listed;
            for (const auto &value : repositories_) {
                const auto row = value.toObject();
                const auto repoId = row.value("repo_id").toString();
                if (!repoId.isEmpty() && !listed.contains(repoId) &&
                    !row.value("archived").toBool()) {
                    listed.insert(repoId);
                    liveAttach->addItem(row.value("display_name").toString(), repoId);
                }
            }
        });
    } else {
        auto *warning = plainLabel(tr("Relinking replaces the saved worktree binding. Pending runs may need a new approval."));
        warning->setWordWrap(true);
        form->addRow(warning);
    }
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Save | QDialogButtonBox::Cancel);
    form->addRow(buttons);
    if (name) {
        auto *nameError = plainLabel(tr("Enter a display name."), "repoNameError");
        form->insertRow(form->rowCount() - 1, nameError);
        const auto validate = [name, attach, nameError, buttons] {
            const bool valid = !name->text().trimmed().isEmpty() || !attach->currentData().toString().isEmpty();
            buttons->button(QDialogButtonBox::Save)->setEnabled(valid);
            nameError->setVisible(!valid);
        };
        connect(name, &QLineEdit::textChanged, &dialog, validate);
        connect(attach, &QComboBox::currentIndexChanged, &dialog, validate);
        validate();
    }
    connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    if (dialog.exec() != QDialog::Accepted) {
        repoActionPending_ = false;
        updateRepositoryActions();
        return;
    }

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
    call(method, params, [this](const QJsonObject &saved, const QString &saveError) {
        repoActionPending_ = false;
        updateRepositoryActions();
        if (!saveError.isEmpty()) {
            showError(saveError);
            return;
        }
        repositorySelectedId_ = bindingId(saved);
        repositoryHealth_.remove(repositorySelectedId_);
        showStatus(tr("Repository binding saved."));
        refreshRepositories();
    });
}

void MainWindow::renameRepository()
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty() || repoActionPending_)
        return;
    bool confirmed = false;
    const auto name = QInputDialog::getText(this, tr("Rename repository"), tr("Display name"),
                                           QLineEdit::Normal, item.value("display_name").toString(), &confirmed);
    if (!confirmed || name.trimmed().isEmpty())
        return;
    repoActionPending_ = true;
    updateRepositoryActions();
    call("repo.rename", {{"repo_id", item.value("repo_id")}, {"revision", item.value("repo_revision")},
                          {"display_name", name.trimmed()}}, [this](const QJsonObject &, const QString &error) {
        repoActionPending_ = false;
        updateRepositoryActions();
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        showStatus(tr("Repository renamed."));
        refreshRepositories();
    });
}

void MainWindow::archiveRepository()
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty() || repoActionPending_)
        return;
    QMessageBox dialog(QMessageBox::Question, tr("Archive repository"),
        tr("Archive %1? The source files and existing history stay on disk. Future dispatches using this repository are blocked.")
            .arg(item.value("display_name").toString()),
        QMessageBox::Yes | QMessageBox::Cancel, this);
    dialog.setTextFormat(Qt::PlainText);
    dialog.setDefaultButton(QMessageBox::Cancel);
    if (dialog.exec() != QMessageBox::Yes)
        return;
    repoActionPending_ = true;
    updateRepositoryActions();
    call("repo.archive", {{"repo_id", item.value("repo_id")}, {"revision", item.value("repo_revision")}},
         [this](const QJsonObject &, const QString &error) {
        repoActionPending_ = false;
        updateRepositoryActions();
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        showStatus(tr("Repository archived."));
        refreshRepositories();
    });
}

void MainWindow::selectRepository()
{
    const auto item = repositorySelection();
    if (!online_ || item.isEmpty() || repoActionPending_)
        return;
    repoActionPending_ = true;
    updateRepositoryActions();
    const auto navigation = navigationRevision_;
    call("repo.select", {{"binding_id", item.value("binding_id")}, {"revision", item.value("revision")}},
         [this, item, navigation](const QJsonObject &value, const QString &error) {
        repoActionPending_ = false;
        updateRepositoryActions();
        if (!error.isEmpty()) {
            showError(error);
            refreshRepositories();
            return;
        }
        if (value.value("status").toString() == QStringLiteral("ACTIVE")) {
            auto health = value;
            health.insert("checked_at", value.value("verified_at"));
            repositoryHealth_.insert(bindingId(item), health);
            refreshRepositories();
            QTimer::singleShot(0, this, [this, id = bindingId(item), navigation] {
                if (online_ && navigation == navigationRevision_ && repositoriesPage_->isVisible())
                    newConversation(id);
            });
        } else {
            showError(tr("The saved binding could not be verified."));
            refreshRepositories();
        }
    });
}
