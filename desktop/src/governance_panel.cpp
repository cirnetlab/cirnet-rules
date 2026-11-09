#include "governance_panel.h"

#include "governance_editor.h"
#include "full_row_table.h"
#include "live_text.h"
#include "theme.h"

#include <QAction>
#include <QCheckBox>
#include <QApplication>
#include <QClipboard>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QFormLayout>
#include <QHeaderView>
#include <QInputDialog>
#include <QIntValidator>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMenu>
#include <QMessageBox>
#include <QPointer>
#include <QPushButton>
#include <QSaveFile>
#include <QResizeEvent>
#include <QScrollArea>
#include <QStyle>
#include <QShortcut>
#include <QShowEvent>
#include <QSplitter>
#include <QStandardItemModel>
#include <QStringDecoder>
#include <QTabWidget>
#include <QTabBar>
#include <QTableView>
#include <QThread>
#include <QTimer>
#include <QUuid>
#include <QVBoxLayout>

namespace {
QString phrase(const char *text)
{
    return QCoreApplication::translate("GovernancePanel", text);
}

QLabel *label(const char *text, const char *role = nullptr)
{
    auto *widget = new QLabel;
    widget->setTextFormat(Qt::PlainText);
    widget->setWordWrap(true);
    if (role)
        widget->setObjectName(QString::fromLatin1(role));
    if (*text)
        live::text(widget, [text] { return phrase(text); });
    return widget;
}

QPushButton *button(const char *text, const char *name, const char *role = "secondary")
{
    auto *widget = live::text(new QPushButton, [text] { return phrase(text); });
    widget->setObjectName(QString::fromLatin1(name));
    if (qstrcmp(role, "primary") == 0)
        widget->setProperty("primary", true);
    else
        widget->setProperty("actionRole", role);
    return widget;
}

QLineEdit *field(const char *name, const char *description)
{
    auto *widget = new QLineEdit;
    widget->setObjectName(QString::fromLatin1(name));
    live::placeholder(widget, [description] { return phrase(description); });
    live::text(widget, [description] { return phrase(description); }, &QWidget::setAccessibleName);
    return widget;
}

QPlainTextEdit *readOnly(const char *name)
{
    auto *widget = new QPlainTextEdit;
    widget->setObjectName(QString::fromLatin1(name));
    widget->setReadOnly(true);
    widget->setProperty("codeText", true);
    widget->setMinimumHeight(100);
    return widget;
}

void tab(QTabWidget *tabs, QWidget *page, const char *title)
{
    const int index = tabs->addTab(page, QString{});
    live::bind(tabs, [tabs, index, title] { tabs->setTabText(index, phrase(title)); });
}

void option(QComboBox *combo, const char *title, const char *key)
{
    live::item(combo, [title] { return phrase(title); }, QString::fromLatin1(key));
}

void kinds(QComboBox *combo)
{
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "Skills"), "skill");
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "Rules"), "rule");
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "Profiles"), "profile");
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "References"), "reference");
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "Policies"), "policy");
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "Commands"), "command");
    option(combo, QT_TRANSLATE_NOOP("GovernancePanel", "Hooks"), "hook");
}

QString pretty(const QJsonValue &value)
{
    if (value.isObject())
        return QString::fromUtf8(QJsonDocument(value.toObject()).toJson());
    if (value.isArray())
        return QString::fromUtf8(QJsonDocument(value.toArray()).toJson());
    return value.toString();
}

QString diffSection(const QString &heading, const QString &before, const QString &after)
{
    if (before == after)
        return {};
    const auto oldLines = before.split(u'\n');
    const auto newLines = after.split(u'\n');
    qsizetype first = 0;
    while (first < oldLines.size() && first < newLines.size() && oldLines[first] == newLines[first])
        ++first;
    auto oldEnd = oldLines.size();
    auto newEnd = newLines.size();
    while (oldEnd > first && newEnd > first && oldLines[oldEnd - 1] == newLines[newEnd - 1]) {
        --oldEnd;
        --newEnd;
    }
    QString output = heading + QStringLiteral("\n@@ -%1,%2 +%1,%3 @@\n")
                                  .arg(first + 1).arg(oldEnd - first).arg(newEnd - first);
    for (auto index = first; index < oldEnd; ++index)
        output += QStringLiteral("- ") + oldLines[index] + u'\n';
    for (auto index = first; index < newEnd; ++index)
        output += QStringLiteral("+ ") + newLines[index] + u'\n';
    return output + u'\n';
}

bool confirm(QWidget *parent, const QString &title, const QString &body)
{
    return QMessageBox::question(parent, title, body, QMessageBox::Yes | QMessageBox::Cancel,
                                 QMessageBox::Cancel) == QMessageBox::Yes;
}
}

GovernancePanel::GovernancePanel(Request request, Repositories repositories)
    : request_(std::move(request)), repositories_(std::move(repositories))
{
    setObjectName("configurationPage");
    setMinimumWidth(0);
    auto *layout = new QVBoxLayout(this);
    layout->setContentsMargins(2, 4, 2, 2);
    layout->setSpacing(8);
    layout->addWidget(label(QT_TRANSLATE_NOOP("GovernancePanel", "Configuration"), "heading"));
    pages_ = new QTabWidget;
    pages_->setObjectName("governanceSections");
    pages_->setDocumentMode(true);
    pages_->tabBar()->setDrawBase(false);
    tab(pages_, makeLibrary(), QT_TRANSLATE_NOOP("GovernancePanel", "Library"));
    tab(pages_, makeAssignments(), QT_TRANSLATE_NOOP("GovernancePanel", "Assignments"));
    tab(pages_, makeIntegrations(), QT_TRANSLATE_NOOP("GovernancePanel", "Integrations"));
    tab(pages_, makeEffective(), QT_TRANSLATE_NOOP("GovernancePanel", "Effective context"));
    layout->addWidget(pages_, 1);
    status_ = label("", "governanceStatus");
    status_->setTextInteractionFlags(Qt::TextSelectableByMouse);
    live::bind(status_, [this] {
        if (statusPhrase_)
            status_->setText(phrase(statusPhrase_));
    });
    layout->addWidget(status_);
    autosave_ = new QTimer(this);
    autosave_->setSingleShot(true);
    autosave_->setInterval(1000);
    connect(autosave_, &QTimer::timeout, this, &GovernancePanel::save);
    filterTimer_ = new QTimer(this);
    filterTimer_->setSingleShot(true);
    filterTimer_->setInterval(200);
    connect(filterTimer_, &QTimer::timeout, this, [this] { refreshCatalog(); });
    connect(search_, &QLineEdit::textChanged, this, [this] { filterTimer_->start(); });
    connect(kind_, &QComboBox::currentIndexChanged, this, [this] { refreshCatalog(); });
    connect(archived_, &QCheckBox::toggled, this, [this] { refreshCatalog(); });
    connect(pages_, &QTabWidget::currentChanged, this, [this](int index) {
        ++scopeGeneration_;
        if (index == 0 && !catalogReady_)
            refreshCatalog();
        if (index == 1 || index == 3)
            refreshScopes();
        if (index == 1)
            refreshAssignments();
        if (index == 2)
            loadIntegrations();
        refreshControls();
    });
    refreshControls();
}

void GovernancePanel::resizeEvent(QResizeEvent *event)
{
    QWidget::resizeEvent(event);
    if (inspectorScroll_)
        layoutLibrary();
}

void GovernancePanel::layoutLibrary()
{
    const bool compact = width() < 620;
    if (compact && !compact_) {
        const auto current = librarySplit_->sizes();
        if (current.value(0) > 0 && current.value(1) > 0)
            wideSizes_ = current;
    }
    catalogPanel_->setVisible(!compact || !inspecting_);
    inspectorScroll_->setVisible(!compact || inspecting_);
    backToCatalog_->setVisible(compact);
    if (compact)
        librarySplit_->setSizes(inspecting_ ? QList<int>{0, width()} : QList<int>{width(), 0});
    else if (compact_)
        librarySplit_->setSizes(wideSizes_);
    compact_ = compact;
}

QWidget *GovernancePanel::makeLibrary()
{
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(0, 6, 0, 0);
    auto *filters = new QHBoxLayout;
    kind_ = new QComboBox;
    option(kind_, QT_TRANSLATE_NOOP("GovernancePanel", "All types"), "");
    kinds(kind_);
    search_ = field("governanceSearch", QT_TRANSLATE_NOOP("GovernancePanel", "Search governance"));
    search_->setProperty("searchField", true);
    search_->setClearButtonEnabled(true);
    search_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Fixed);
    search_->setMinimumWidth(50);
    search_->setMaxLength(96);
    filters->addWidget(kind_);
    filters->addWidget(search_, 1);
    new_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "New asset"), "governanceNew", "primary");
    new_->setIcon(uiIcon(UiIcon::Add));
    import_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Import file"), "governanceImport");
    filters->addWidget(import_);
    filters->addWidget(new_);
    layout->addLayout(filters);

    librarySplit_ = new QSplitter(Qt::Horizontal);
    librarySplit_->setChildrenCollapsible(false);
    catalogPanel_ = new QWidget;
    catalogPanel_->setMinimumWidth(145);
    auto *catalogLayout = new QVBoxLayout(catalogPanel_);
    catalogLayout->setContentsMargins(0, 0, 0, 0);
    auto *options = new QHBoxLayout;
    archived_ = live::text(new QCheckBox, [] { return tr("Include archived"); });
    options->addWidget(archived_);
    options->addStretch();
    auto *refresh = button(QT_TRANSLATE_NOOP("GovernancePanel", "Reload"), "governanceReload", "ghost");
    options->addWidget(refresh);
    catalogLayout->addLayout(options);

    catalog_ = new FullRowTable;
    catalog_->setObjectName("governanceCatalog");
    live::text(catalog_, [] { return tr("Governance library"); }, &QWidget::setAccessibleName);
    catalog_->verticalHeader()->hide();
    catalog_->horizontalHeader()->setStretchLastSection(true);
    catalog_->horizontalHeader()->setSectionResizeMode(QHeaderView::Interactive);
    model_ = new QStandardItemModel(0, 2, catalog_);
    live::bind(catalog_, [this] { model_->setHorizontalHeaderLabels({tr("Asset"), tr("Revision")}); });
    catalog_->setModel(model_);
    catalog_->setColumnWidth(0, 155);
    catalogLayout->addWidget(catalog_, 1);
    more_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Next page"), "governanceMore");
    catalogLayout->addWidget(more_, 0, Qt::AlignRight);
    librarySplit_->addWidget(catalogPanel_);
    inspectorScroll_ = new QScrollArea;
    inspectorScroll_->setObjectName("governanceInspector");
    inspectorScroll_->setWidgetResizable(true);
    inspectorScroll_->setFrameShape(QFrame::NoFrame);
    inspectorScroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    inspectorScroll_->setWidget(makeInspector());
    librarySplit_->addWidget(inspectorScroll_);
    librarySplit_->setStretchFactor(0, 0);
    librarySplit_->setStretchFactor(1, 1);
    librarySplit_->setSizes({240, 760});
    layout->addWidget(librarySplit_, 1);

    connect(refresh, &QPushButton::clicked, this, [this] { refreshCatalog(); });
    connect(more_, &QPushButton::clicked, this, [this] { refreshCatalog(true); });
    connect(new_, &QPushButton::clicked, this, [this] { createAsset(); });
    auto *importMenu = new QMenu(import_);
    auto *textImport = importMenu->addAction(tr("Import UTF-8 text"));
    auto *bundleImport = importMenu->addAction(tr("Import published bundle"));
    live::text(textImport, [] { return tr("Import UTF-8 text"); });
    live::text(bundleImport, [] { return tr("Import published bundle"); });
    import_->setMenu(importMenu);
    connect(textImport, &QAction::triggered, this, [this] {
        readImport([this](const QString &name, const QString &source) { createAsset(name, source); });
    });
    connect(bundleImport, &QAction::triggered, this, [this] {
        if (abandonEdits())
            readImport([this](const QString &, const QString &source) { importBundle(source); });
    });
    connect(catalog_->selectionModel(), &QItemSelectionModel::currentRowChanged, this,
            [this](const QModelIndex &index) {
        if (!loading_ && index.isValid())
            openAsset(index.siblingAtColumn(0).data(Qt::UserRole).toString());
    });
    connect(catalog_, &QTableView::clicked, this, [this](const QModelIndex &index) {
        const auto id = index.siblingAtColumn(0).data(Qt::UserRole).toString();
        if (!inspecting_ && !id.isEmpty() && id == selectedId_)
            openAsset(id);
    });
    return page;
}

QWidget *GovernancePanel::makeInspector()
{
    auto *panel = new QWidget;
    panel->setMinimumWidth(310);
    auto *layout = new QVBoxLayout(panel);
    layout->setContentsMargins(4, 0, 2, 0);
    backToCatalog_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Back to assets"), "governanceBack", "ghost");
    backToCatalog_->setIcon(uiIcon(UiIcon::Back));
    backToCatalog_->hide();
    layout->addWidget(backToCatalog_, 0, Qt::AlignLeft);
    connect(backToCatalog_, &QPushButton::clicked, this, [this] {
        inspecting_ = false;
        layoutLibrary();
        catalog_->setFocus();
    });
    assetState_ = label("", "lead");
    live::bind(assetState_, [this] { updateAssetState(); });
    layout->addWidget(assetState_);
    auto *metadata = new QGridLayout;
    title_ = field("governanceTitle", QT_TRANSLATE_NOOP("GovernancePanel", "Title"));
    title_->setMaxLength(160);
    description_ = field("governanceDescription", QT_TRANSLATE_NOOP("GovernancePanel", "Description"));
    description_->setMaxLength(2048);
    dependencies_ = field("governanceDependencies", QT_TRANSLATE_NOOP("GovernancePanel", "Dependency IDs, separated by commas"));
    dependencies_->setMaxLength(4096);
    format_ = new QComboBox;
    option(format_, "Markdown", "markdown");
    option(format_, "YAML", "yaml");
    option(format_, "JSON", "json");
    metadata->addWidget(title_, 0, 0);
    metadata->addWidget(format_, 0, 1);
    metadata->addWidget(description_, 1, 0, 1, 2);
    metadata->addWidget(dependencies_, 2, 0, 1, 2);
    layout->addLayout(metadata);

    contentTabs_ = new QTabWidget;
    contentTabs_->setObjectName("governanceEditorTabs");
    contentTabs_->setDocumentMode(true);
    contentTabs_->tabBar()->setDrawBase(false);
    english_ = new GovernanceEditor;
    english_->setObjectName("governanceEnglish");
    english_->setAccessibleName(QStringLiteral("EN"));
    vietnamese_ = new GovernanceEditor;
    vietnamese_->setObjectName("governanceVietnamese");
    vietnamese_->setAccessibleName(QStringLiteral("VI"));
    tab(contentTabs_, english_, QT_TRANSLATE_NOOP("GovernancePanel", "EN · Canonical"));
    tab(contentTabs_, vietnamese_, QT_TRANSLATE_NOOP("GovernancePanel", "VI · Translation"));
    auto *files = new QWidget;
    auto *filesLayout = new QVBoxLayout(files);
    fileList_ = new QListWidget;
    fileList_->setObjectName("governanceFiles");
    filesLayout->addWidget(fileList_, 1);
    auto *fileActions = new QHBoxLayout;
    auto *add = button(QT_TRANSLATE_NOOP("GovernancePanel", "Add file"), "governanceAddFile");
    auto *attach = button(QT_TRANSLATE_NOOP("GovernancePanel", "Attach text"), "governanceAttach");
    auto *open = button(QT_TRANSLATE_NOOP("GovernancePanel", "Edit file"), "governanceEditFile");
    auto *remove = button(QT_TRANSLATE_NOOP("GovernancePanel", "Remove"), "governanceRemoveFile");
    for (auto *item : {add, attach, open, remove})
        fileActions->addWidget(item);
    filesLayout->addLayout(fileActions);
    connect(add, &QPushButton::clicked, this, [this] { editAttachment(true); });
    connect(open, &QPushButton::clicked, this, [this] { editAttachment(false); });
    connect(fileList_, &QListWidget::itemDoubleClicked, this, [this] { editAttachment(false); });
    connect(attach, &QPushButton::clicked, this, [this] {
        readImport([this](const QString &name, const QString &source) {
            files_.append(QJsonObject{{"path", name}, {"content", source}});
            fileList_->addItem(name);
            edited();
        });
    });
    connect(remove, &QPushButton::clicked, this, [this] {
        const auto row = fileList_->currentRow();
        if (row >= 0) {
            files_.removeAt(row);
            delete fileList_->takeItem(row);
            edited();
        }
    });
    tab(contentTabs_, files, QT_TRANSLATE_NOOP("GovernancePanel", "Files"));
    auto *previewPage = new QWidget;
    auto *previewLayout = new QVBoxLayout(previewPage);
    previewLayout->setContentsMargins(0, 2, 0, 0);
    previewLocale_ = new QComboBox;
    previewLocale_->addItems({QStringLiteral("EN"), QStringLiteral("VI")});
    live::text(previewLocale_, [] { return tr("Preview language"); }, &QWidget::setAccessibleName);
    previewLayout->addWidget(previewLocale_, 0, Qt::AlignRight);
    preview_ = new GovernancePreview;
    previewLayout->addWidget(preview_, 1);
    tab(contentTabs_, previewPage, QT_TRANSLATE_NOOP("GovernancePanel", "Preview"));
    connect(previewLocale_, &QComboBox::currentIndexChanged, this, &GovernancePanel::updateEditorPreview);
    diff_ = readOnly("governanceDiff");
    tab(contentTabs_, diff_, QT_TRANSLATE_NOOP("GovernancePanel", "Diff"));
    layout->addWidget(contentTabs_, 1);
    connect(contentTabs_, &QTabWidget::currentChanged, this, &GovernancePanel::updateEditorPreview);
    for (auto *input : {title_, description_, dependencies_})
        connect(input, &QLineEdit::textEdited, this, &GovernancePanel::edited);
    connect(format_, &QComboBox::currentIndexChanged, this, &GovernancePanel::edited);
    connect(english_, &QPlainTextEdit::textChanged, this, &GovernancePanel::edited);
    connect(vietnamese_, &QPlainTextEdit::textChanged, this, &GovernancePanel::edited);
    diagnostics_ = label("", "governanceDiagnostics");
    diagnostics_->setTextInteractionFlags(Qt::TextSelectableByMouse);
    diagnostics_->hide();
    layout->addWidget(diagnostics_);

    auto *actions = new QGridLayout;
    save_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Save draft"), "governanceSave");
    validate_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Validate"), "governanceValidate");
    compare_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "View diff"), "governanceCompare");
    review_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Review"), "governanceReview");
    apply_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Apply"), "governanceApply", "primary");
    history_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "History"), "governanceHistory");
    actions_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Asset actions"), "governanceActions", "ghost");
    actions->addWidget(save_, 0, 0);
    actions->addWidget(validate_, 0, 1);
    actions->addWidget(compare_, 0, 2);
    actions->addWidget(review_, 1, 0);
    actions->addWidget(apply_, 1, 1);
    actions->addWidget(history_, 1, 2);
    actions->addWidget(actions_, 2, 0, 1, 3, Qt::AlignRight);
    layout->addLayout(actions);
    connect(save_, &QPushButton::clicked, this, &GovernancePanel::save);
    connect(validate_, &QPushButton::clicked, this, &GovernancePanel::validate);
    connect(compare_, &QPushButton::clicked, this, [this] { compare(false); });
    connect(review_, &QPushButton::clicked, this, [this] { compare(true); });
    connect(apply_, &QPushButton::clicked, this, &GovernancePanel::publish);
    connect(history_, &QPushButton::clicked, this, &GovernancePanel::showHistory);
    connect(actions_, &QPushButton::clicked, this, [this] {
        QMenu menu(this);
        const auto add = [this, &menu](const QString &title, const QString &action) {
            connect(menu.addAction(title), &QAction::triggered, this, [this, action] { changeAsset(action); });
        };
        auto *copy = menu.addAction(tr("Copy asset ID"));
        connect(copy, &QAction::triggered, this, [this] {
            QApplication::clipboard()->setText(selectedId_);
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Asset ID copied."));
        });
        auto *inspect = menu.addAction(tr("Inspect identity and consumers"));
        connect(inspect, &QAction::triggered, this, [this] {
            QMessageBox box(QMessageBox::Information, tr("Asset identity"), {}, QMessageBox::Ok, this);
            box.setTextFormat(Qt::PlainText);
            box.setText(tr("ID: %1\nSource: %2\nConsumers: %3\nBindings: %4")
                .arg(selectedId_, asset_.value("source_name").toString(tr("Local authoring")),
                     pretty(asset_.value("used_by")).trimmed(), pretty(asset_.value("assigned_bindings")).trimmed()));
            box.exec();
        });
        add(tr("Reload saved revision"), "asset.reload");
        if (asset_.value("active_revision").isDouble())
            add(tr("Export published revision"), "asset.export");
        add(tr("Rename"), "asset.rename");
        add(tr("Duplicate"), "asset.duplicate");
        if (asset_.value("kind").toString() == "profile" && asset_.value("active_revision").isDouble()
            && !asset_.value("archived").toBool())
            add(asset_.value("default_profile").toBool() ? tr("Clear default profile") : tr("Use for new chats"),
                "profile.default");
        if (asset_.value("draft").toBool())
            add(tr("Discard saved draft"), "draft.discard");
        if (asset_.value("archived").toBool())
            add(tr("Restore asset"), "asset.restore");
        else
            add(tr("Archive asset"), "asset.archive");
        add(tr("Delete asset"), "asset.delete");
        menu.exec(actions_->mapToGlobal(QPoint(0, actions_->height())));
    });

    auto *saveShortcut = new QShortcut(QKeySequence::Save, panel);
    saveShortcut->setContext(Qt::WidgetWithChildrenShortcut);
    connect(saveShortcut, &QShortcut::activated, this, &GovernancePanel::save);
    auto *find = new QShortcut(QKeySequence::Find, panel);
    find->setContext(Qt::WidgetWithChildrenShortcut);
    connect(find, &QShortcut::activated, this, [this] {
        auto *editor = qobject_cast<QPlainTextEdit *>(contentTabs_->currentWidget());
        if (!editor || editor->isReadOnly() || !editor->isEnabled())
            return;
        QDialog dialog(this);
        dialog.setWindowTitle(tr("Find and replace"));
        auto *form = new QFormLayout(&dialog);
        auto *query = field("governanceFind", QT_TRANSLATE_NOOP("GovernancePanel", "Find text"));
        auto *replacement = field("governanceReplace", QT_TRANSLATE_NOOP("GovernancePanel", "Replacement"));
        auto *findNext = button(QT_TRANSLATE_NOOP("GovernancePanel", "Find next"), "governanceFindNext");
        auto *replace = button(QT_TRANSLATE_NOOP("GovernancePanel", "Replace selection"), "governanceReplaceSelected");
        form->addRow(query);
        form->addRow(replacement);
        form->addRow(findNext, replace);
        auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close);
        form->addRow(buttons);
        connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        connect(findNext, &QPushButton::clicked, &dialog, [editor, query] {
            if (!query->text().isEmpty() && !editor->find(query->text())) {
                editor->moveCursor(QTextCursor::Start);
                editor->find(query->text());
            }
        });
        connect(replace, &QPushButton::clicked, &dialog, [editor, query, replacement] {
            auto cursor = editor->textCursor();
            if (!query->text().isEmpty() && cursor.selectedText() == query->text())
                cursor.insertText(replacement->text());
        });
        dialog.exec();
    });
    return panel;
}


void GovernancePanel::showEvent(QShowEvent *event)
{
    QWidget::showEvent(event);
    if (!catalogReady_ && pages_->currentIndex() == 0)
        refreshCatalog();
    repositoryCatalogChanged();
    if (pages_->currentIndex() == 2 && adapterItems_.isEmpty())
        loadIntegrations();
}

void GovernancePanel::setAvailable(bool available)
{
    if (online_ == available)
        return;
    online_ = available;
    if (!online_) {
        ++catalogGeneration_;
        ++assetGeneration_;
        ++scopeGeneration_;
        catalogLoading_ = false;
        assetLoading_ = false;
        catalogReady_ = false;
        reviewToken_.clear();
        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Governance runtime disconnected. Unsaved text is still in the editor."), true);
    } else {
        if (isVisible()) {
            if (pages_->currentIndex() == 0)
                refreshCatalog();
            if (pages_->currentIndex() == 1)
                refreshAssignments();
            if (pages_->currentIndex() == 2)
                loadIntegrations();
        }
        if (dirty_)
            autosave_->start();
    }
    refreshControls();
}

bool GovernancePanel::confirmClose()
{
    if (writing_) {
        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Wait for the governance operation to finish before closing."), true);
        return false;
    }
    return abandonEdits();
}

bool GovernancePanel::abandonEdits()
{
    if (writing_)
        return false;
    if (!dirty_)
        return true;
    autosave_->stop();
    const auto abandon = confirm(this, tr("Unsaved governance draft"),
                                 tr("Continue without saving changes to this draft?"));
    if (!abandon && online_)
        autosave_->start();
    return abandon;
}

void GovernancePanel::report(const QString &message, bool failure)
{
    statusPhrase_ = nullptr;
    status_->setText(message);
    status_->setProperty("governanceError", failure);
    status_->style()->unpolish(status_);
    status_->style()->polish(status_);
}

void GovernancePanel::reportStatic(const char *message, bool failure)
{
    report(phrase(message), failure);
    statusPhrase_ = message;
}

void GovernancePanel::send(const QString &method, QJsonObject params, RuntimeClient::Reply reply)
{
    const QPointer<GovernancePanel> self(this);
    request_(method, std::move(params), [self, reply = std::move(reply)](const QJsonObject &result, const QString &error) {
        if (self && reply)
            reply(result, error);
    });
}

void GovernancePanel::mutate(const QString &method, QJsonObject params, RuntimeClient::Reply reply)
{
    if (!online_ || writing_)
        return;
    auto previous = pendingMutation_;
    const auto previousKey = previous.take("mutation_key");
    const auto key = method == pendingMethod_ && params == previous && previousKey.isString()
        ? previousKey.toString() : QUuid::createUuid().toString(QUuid::WithoutBraces);
    params.insert("mutation_key", key);
    pendingMethod_ = method;
    pendingMutation_ = params;
    writing_ = true;
    autosave_->stop();
    refreshControls();
    send("governance." + method, std::move(params), [this, reply = std::move(reply)](const auto &result, const auto &error) {
        writing_ = false;
        if (error.isEmpty()) {
            pendingMutation_ = {};
            pendingMethod_.clear();
        } else {
            const auto code = result.value("code").toString();
            report(code.isEmpty() ? error : uiStatusLabel(code) + QStringLiteral(" · ") + error, true);
        }
        if (reply)
            reply(result, error);
        refreshControls();
    });
}

void GovernancePanel::refreshCatalog(bool append)
{
    if (!online_ || (append && (nextPage_.isEmpty() || catalogLoading_)))
        return;
    const auto serial = ++catalogGeneration_;
    catalogLoading_ = true;
    refreshControls();
    QJsonObject query{{"search", search_->text()}, {"include_archived", archived_->isChecked()}, {"limit", 50}};
    const auto kind = kind_->currentData().toString();
    if (!kind.isEmpty())
        query.insert("kind", kind);
    if (append)
        query.insert("after", nextPage_);
    send("governance.asset.list", query, [this, serial, append](const auto &result, const auto &error) {
        if (serial != catalogGeneration_)
            return;
        catalogLoading_ = false;
        if (!error.isEmpty()) {
            report(error, true);
            refreshControls();
            return;
        }
        catalogReady_ = true;
        const QSignalBlocker selectionSignals(catalog_->selectionModel());
        if (!append && model_->rowCount())
            model_->removeRows(0, model_->rowCount());
        for (const auto &value : result.value("items").toArray()) {
            const auto item = value.toObject();
            auto *title = new QStandardItem(item.value("title").toString());
            title->setData(item.value("id").toString(), Qt::UserRole);
            const auto state = item.value("archived").toBool() ? tr("Archived") :
                item.value("draft").toBool() ? tr("Draft") : tr("Published");
            title->setToolTip(tr("%1 · %2\n%3").arg(uiStatusLabel(item.value("kind").toString()), state,
                                                   item.value("description").toString()).toHtmlEscaped());
            auto *revision = new QStandardItem(item.value("active_revision").isDouble()
                ? tr("r%1").arg(item.value("active_revision").toInteger()) : state);
            model_->appendRow({title, revision});
        }
        nextPage_ = result.value("next_after").toString();
        updateSelection();
        if (!model_->rowCount() && asset_.isEmpty())
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "No governance assets match the current filters."));
        refreshControls();
    });
}

void GovernancePanel::updateSelection()
{
    const QSignalBlocker selectionBlock(catalog_->selectionModel());
    for (int row = 0; row < model_->rowCount(); ++row) {
        const auto index = model_->index(row, 0);
        if (index.data(Qt::UserRole).toString() == selectedId_) {
            catalog_->selectRow(row);
            return;
        }
    }
    catalog_->clearSelection();
    catalog_->setCurrentIndex({});
}

void GovernancePanel::openAsset(const QString &id)
{
    if (!id.isEmpty() && id == selectedId_) {
        inspecting_ = true;
        layoutLibrary();
        return;
    }
    if (id.isEmpty() || !online_ || !abandonEdits()) {
        updateSelection();
        return;
    }
    assetLoading_ = true;
    refreshControls();
    const auto serial = ++assetGeneration_;
    send("governance.asset.get", {{"id", id}}, [this, serial](const auto &result, const auto &error) {
        if (serial != assetGeneration_)
            return;
        assetLoading_ = false;
        if (error.isEmpty()) {
            setAsset(result);
            report({});
        } else {
            updateSelection();
            report(error, true);
        }
        refreshControls();
    });
}

void GovernancePanel::setAsset(const QJsonObject &asset)
{
    const auto changed = selectedId_ != asset.value("id").toString();
    loading_ = true;
    asset_ = asset;
    selectedId_ = asset.value("id").toString();
    if (changed) {
        inspecting_ = !selectedId_.isEmpty();
        layoutLibrary();
    }
    const auto doc = asset.value("document").toObject();
    if (english_->toPlainText() != doc.value("en").toString())
        english_->setPlainText(doc.value("en").toString());
    if (vietnamese_->toPlainText() != doc.value("vi").toString())
        vietnamese_->setPlainText(doc.value("vi").toString());
    title_->setText(asset.value("title").toString());
    description_->setText(doc.value("description").toString(asset.value("description").toString()));
    QStringList dependencies;
    for (const auto &value : doc.value("dependencies").toArray())
        dependencies.append(value.toString());
    dependencies_->setText(dependencies.join(QStringLiteral(", ")));
    const auto formatIndex = format_->findData(doc.value("format").toString());
    format_->setCurrentIndex(qMax(0, formatIndex));
    const auto attachments = doc.value("files").toArray();
    if (files_ != attachments) {
        files_ = attachments;
        fileList_->clear();
        for (const auto &value : files_)
            fileList_->addItem(value.toObject().value("path").toString());
    }
    if (changed) {
        contentTabs_->setCurrentIndex(0);
        diff_->clear();
    }
    updateAssetState();
    diagnostics_->hide();
    autosave_->stop();
    dirty_ = false;
    reviewToken_.clear();
    loading_ = false;
    updateSelection();
    updateEditorPreview();
    refreshControls();
}

void GovernancePanel::updateAssetState()
{
    if (asset_.isEmpty()) {
        assetState_->setText(tr("Select an asset or create a draft."));
        assetState_->setToolTip({});
        return;
    }
    const auto translation = asset_.value("translation").toString();
    const auto state = translation == "CURRENT" ? tr("Current")
                     : translation == "STALE" ? tr("Stale")
                     : translation == "MISSING" ? tr("Missing")
                     : tr("Unverified");
    const auto origin = asset_.value("origin").toString() == "local" ? tr("Local authoring") : tr("Imported");
    assetState_->setText(tr("%1 · %2 · r%3 · Translation: %4")
        .arg(asset_.value("slug").toString(), origin)
        .arg(asset_.value("revision").toInteger()).arg(state));
    const auto used = asset_.value("used_by").toArray();
    assetState_->setToolTip(used.isEmpty() ? QString{} : tr("Used by:\n%1").arg(pretty(used)).toHtmlEscaped());
}

QJsonObject GovernancePanel::document() const
{
    QJsonArray dependencies;
    for (const auto &dependency : dependencies_->text().split(u',', Qt::SkipEmptyParts)) {
        const auto value = dependency.trimmed();
        if (!value.isEmpty())
            dependencies.append(value);
    }
    return {{"en", english_->toPlainText()}, {"vi", vietnamese_->toPlainText()},
            {"format", format_->currentData().toString()}, {"files", files_},
            {"dependencies", dependencies}, {"title", title_->text()}, {"description", description_->text()}};
}

QJsonObject GovernancePanel::versioned() const
{
    return {{"id", asset_.value("id")}, {"expected_revision", asset_.value("revision")}};
}

void GovernancePanel::edited()
{
    if (loading_ || asset_.isEmpty() || asset_.value("archived").toBool())
        return;
    dirty_ = true;
    reviewToken_.clear();
    diagnostics_->hide();
    reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Draft has unsaved changes."));
    if (online_)
        autosave_->start();
    refreshControls();
}

void GovernancePanel::refreshControls()
{
    const auto selected = !asset_.isEmpty() && !assetLoading_;
    const auto editable = selected && !writing_ && !asset_.value("archived").toBool();
    const auto saved = online_ && editable && !dirty_ && asset_.value("draft").toBool();
    new_->setEnabled(online_ && !writing_);
    import_->setEnabled(online_ && !writing_);
    catalog_->setEnabled(!writing_);
    more_->setVisible(!nextPage_.isEmpty());
    more_->setEnabled(online_ && !catalogLoading_);
    for (auto *input : {title_, description_, dependencies_})
        input->setEnabled(editable);
    format_->setEnabled(editable);
    english_->setEnabled(selected);
    vietnamese_->setEnabled(selected);
    english_->setReadOnly(!editable);
    vietnamese_->setReadOnly(!editable);
    fileList_->parentWidget()->setEnabled(editable);
    save_->setEnabled(online_ && editable && dirty_);
    validate_->setEnabled(saved);
    compare_->setEnabled(saved);
    review_->setEnabled(saved);
    apply_->setEnabled(saved && !reviewToken_.isEmpty());
    history_->setEnabled(online_ && selected && !writing_ && !dirty_ && asset_.value("active_revision").isDouble());
    actions_->setEnabled(online_ && selected && !writing_);
    assign_->setEnabled(online_ && selected && !writing_ && !dirty_ && !assignmentBinding_->currentData().toString().isEmpty()
                        && asset_.value("active_revision").isDouble() && !asset_.value("archived").toBool());
    unassign_->setEnabled(online_ && selected && !writing_ && !dirty_ && !assignmentBinding_->currentData().toString().isEmpty());
    probe_->setEnabled(online_ && !integrationLoading_ && !adapter_->currentData().toString().isEmpty());
    inspectImport_->setEnabled(online_ && !writing_ && !adapter_->currentData().toString().isEmpty());
}

void GovernancePanel::save()
{
    if (!save_->isEnabled())
        return;
    auto params = document();
    const auto revision = versioned();
    for (auto it = revision.begin(); it != revision.end(); ++it)
        params.insert(it.key(), it.value());
    mutate("draft.save", params, [this](const auto &result, const auto &error) {
        if (!error.isEmpty())
            return;
        const bool searchAffected = !search_->text().isEmpty() &&
            (result.value("title") != asset_.value("title") || result.value("description") != asset_.value("description"));
        setAsset(result);
        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Draft saved. The published revision is unchanged."));
        if (searchAffected)
            refreshCatalog();
        else {
            for (int row = 0; row < model_->rowCount(); ++row) {
                auto *entry = model_->item(row, 0);
                if (entry->data(Qt::UserRole).toString() != selectedId_)
                    continue;
                entry->setText(asset_.value("title").toString());
                entry->setToolTip(asset_.value("description").toString().toHtmlEscaped());
                break;
            }
        }
    });
}

void GovernancePanel::updateDiagnostics(const QJsonObject &result)
{
    QStringList lines;
    for (const auto &value : result.value("diagnostics").toArray()) {
        const auto entry = value.toObject();
        auto location = uiStatusLabel(entry.value("field").toString());
        if (entry.value("line").isDouble())
            location += QStringLiteral(" %1:%2").arg(entry.value("line").toInt()).arg(entry.value("column").toInt());
        lines.append(tr("%1 · %2").arg(location, entry.value("message").toString()));
        if (lines.size() == 5)
            break;
    }
    if (lines.isEmpty())
        lines.append(result.value("valid").toBool() ? tr("Validation passed.") : tr("Validation failed."));
    const auto translation = result.value("translation").toString();
    if (!translation.isEmpty())
        lines.append(tr("Translation: %1").arg(uiStatusLabel(translation)));
    diagnostics_->setText(lines.join(u'\n'));
    diagnostics_->show();
}

void GovernancePanel::validate()
{
    const auto expected = versioned();
    send("governance.revision.validate", expected, [this, expected](const auto &result, const auto &error) {
        if (expected != versioned() || dirty_)
            return;
        if (error.isEmpty())
            updateDiagnostics(result);
        else
            report(error, true);
    });
}

void GovernancePanel::compare(bool review)
{
    const auto expected = versioned();
    send("governance.revision.compare", expected, [this, expected, review](const auto &result, const auto &error) {
        if (expected != versioned() || dirty_)
            return;
        if (!error.isEmpty()) {
            report(error, true);
            return;
        }
        updateDiagnostics(result);
        const auto render = [this, expected, review, valid = result.value("valid").toBool()]
            (const QJsonObject &before, const QJsonObject &after) {
            if (expected != versioned() || dirty_)
                return;
            auto text = diffSection("EN", before.value("en").toString(), after.value("en").toString());
            text += diffSection("VI", before.value("vi").toString(), after.value("vi").toString());
            auto oldMetadata = before;
            auto newMetadata = after;
            for (const auto &key : {"en", "vi"}) {
                oldMetadata.remove(key);
                newMetadata.remove(key);
            }
            text += diffSection(tr("Metadata and files"), pretty(oldMetadata), pretty(newMetadata));
            diff_->setPlainText(text.isEmpty() ? tr("No content changes.") : text);
            contentTabs_->setCurrentWidget(diff_);
            if (review && valid)
                mutate("change.review", expected, [this](const auto &receipt, const auto &failure) {
                    if (failure.isEmpty()) {
                        reviewToken_ = receipt.value("review_token").toString();
                        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Review ready. Apply explicitly to activate this revision."));
                    }
                });
        };
        if (!result.value("details_truncated").toBool()) {
            render(result.value("active").toObject(), result.value("draft").toObject());
            return;
        }
        const auto after = asset_.value("document").toObject();
        const auto active = asset_.value("active_revision");
        if (!active.isDouble()) {
            render({}, after);
            return;
        }
        send("governance.revision.get", {{"id", asset_.value("id")}, {"revision", active}},
             [this, expected, after, render](const auto &revision, const auto &failure) {
            if (expected != versioned() || dirty_)
                return;
            if (failure.isEmpty())
                render(revision.value("document").toObject(), after);
            else
                report(failure, true);
        });
    });
}

void GovernancePanel::publish()
{
    if (!apply_->isEnabled() ||
        !confirm(this, tr("Apply reviewed revision"),
                 tr("Activate the reviewed revision for future context previews? Existing history will be preserved.")))
        return;
    auto params = versioned();
    params.insert("review_token", reviewToken_);
    mutate("change.publish", params, [this](const auto &result, const auto &error) {
        if (error.isEmpty()) {
            setAsset(result);
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Reviewed revision published."));
            refreshCatalog();
        }
    });
}

void GovernancePanel::updateEditorPreview()
{
    if (contentTabs_->currentWidget() == preview_->parentWidget()) {
        const auto *editor = previewLocale_->currentIndex() == 0 ? english_ : vietnamese_;
        preview_->showSource(editor->toPlainText(), format_->currentData() == "markdown");
    }
}


void GovernancePanel::createAsset(const QString &sourceName, const QString &source)
{
    if (!abandonEdits())
        return;
    QDialog dialog(this);
    dialog.setWindowTitle(sourceName.isEmpty() ? tr("New governance asset") : tr("Import a local draft"));
    auto *form = new QFormLayout(&dialog);
    auto *kind = new QComboBox;
    kinds(kind);
    const auto preferredKind = sourceName.compare("SKILL.md", Qt::CaseInsensitive) == 0
        ? QStringLiteral("skill") : kind_->currentData().toString();
    kind->setCurrentIndex(qMax(0, kind->findData(preferredKind)));
    auto *name = field("governanceNewTitle", QT_TRANSLATE_NOOP("GovernancePanel", "Title"));
    auto *slug = field("governanceNewSlug", QT_TRANSLATE_NOOP("GovernancePanel", "Stable slug"));
    name->setMaxLength(160);
    slug->setMaxLength(100);
    form->addRow(tr("Type"), kind);
    form->addRow(tr("Title"), name);
    form->addRow(tr("Slug"), slug);
    if (!sourceName.isEmpty()) {
        name->setText(QFileInfo(sourceName).completeBaseName());
        auto *sourceInfo = new QLabel(tr("Source: %1. Imported text becomes a local draft. It is never executed.").arg(sourceName));
        sourceInfo->setTextFormat(Qt::PlainText);
        sourceInfo->setWordWrap(true);
        form->addRow(sourceInfo);
        auto *sample = readOnly("governanceImportSample");
        sample->setPlainText(source.left(4096));
        form->addRow(sample);
    }
    auto *actions = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
    form->addRow(actions);
    connect(actions, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(actions, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    if (dialog.exec() != QDialog::Accepted)
        return;
    const auto suffix = QFileInfo(sourceName).suffix().toLower();
    const auto format = suffix == "json" ? "json" : suffix == "yaml" || suffix == "yml" ? "yaml" : "markdown";
    QJsonObject params{{"kind", kind->currentData().toString()}, {"title", name->text().trimmed()},
                       {"slug", slug->text().trimmed()}};
    if (!sourceName.isEmpty())
        params.insert("format", format);
    if (!sourceName.isEmpty())
        params.insert("source_name", sourceName);
    ++assetGeneration_;
    assetLoading_ = false;
    mutate("asset.create", params, [this, sourceName, source](const auto &result, const auto &error) {
        if (!error.isEmpty())
            return;
        setAsset(result);
        if (sourceName.isEmpty()) {
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Draft created."));
        } else {
            english_->setPlainText(source);
            edited();
            save();
        }
        refreshCatalog();
    });
}

void GovernancePanel::changeAsset(const QString &action)
{
    if (!abandonEdits())
        return;
    if (action == "asset.export") {
        const auto expected = versioned();
        send("governance.asset.export", {{"id", expected.value("id")}}, [this, expected](const auto &result, const auto &error) {
            if (expected != versioned())
                return;
            if (!error.isEmpty()) {
                report(error, true);
                return;
            }
            QDialog dialog(this);
            dialog.setWindowTitle(tr("Review published export"));
            auto *layout = new QVBoxLayout(&dialog);
            layout->addWidget(label(QT_TRANSLATE_NOOP("GovernancePanel", "Review the published content before exporting it.")));
            auto *preview = readOnly("governanceExportPreview");
            preview->setPlainText(pretty(result));
            layout->addWidget(preview);
            auto *actions = new QDialogButtonBox(QDialogButtonBox::Save | QDialogButtonBox::Cancel);
            connect(actions, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
            connect(actions, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
            layout->addWidget(actions);
            dialog.resize(620, 460);
            if (dialog.exec() != QDialog::Accepted)
                return;
            const auto path = QFileDialog::getSaveFileName(this, tr("Save published export"),
                asset_.value("slug").toString() + ".json", tr("JSON (*.json)"));
            if (path.isEmpty())
                return;
            QSaveFile output(path);
            const auto bytes = QJsonDocument(result).toJson();
            if (output.open(QIODevice::WriteOnly) && output.write(bytes) == bytes.size() && output.commit())
                reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Published export saved."));
            else
                reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Could not save the published export."), true);
        });
        return;
    }
    auto params = versioned();
    if (action == "profile.default") {
        const bool enabled = !asset_.value("default_profile").toBool();
        if (!confirm(this, tr("Change the default profile"),
                     tr("Apply this choice to new chats? Existing conversations keep their pinned revisions.")))
            return;
        params.insert("enabled", enabled);
        mutate(action, params, [this](const auto &result, const auto &error) {
            if (error.isEmpty()) {
                setAsset(result);
                reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Default profile updated for new chats."));
            }
        });
        return;
    }
    if (action == "asset.rename" || action == "asset.duplicate") {
        const bool duplicate = action == "asset.duplicate";
        QDialog dialog(this);
        dialog.setWindowTitle(duplicate ? tr("Duplicate asset") : tr("Rename asset"));
        auto *form = new QFormLayout(&dialog);
        auto *title = new QLineEdit(asset_.value("title").toString() + (duplicate ? tr(" copy") : QString{}));
        auto *slug = new QLineEdit(asset_.value("slug").toString() + (duplicate ? QStringLiteral("-copy") : QString{}));
        title->setMaxLength(160);
        slug->setMaxLength(100);
        form->addRow(tr("Title"), title);
        form->addRow(tr("Slug"), slug);
        auto *actions = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
        connect(actions, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
        connect(actions, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        form->addRow(actions);
        if (dialog.exec() != QDialog::Accepted)
            return;
        params.insert("slug", slug->text().trimmed());
        params.insert("title", title->text().trimmed());
    } else if (action == "asset.delete") {
        if (!confirm(this, tr("Remove archived asset"), tr("Remove this archived asset from the library? "
                     "Published revisions remain in the audit record. Dependencies prevent removal.")))
            return;
    } else if (action == "asset.reload") {
        const auto id = selectedId_;
        selectedId_.clear();
        dirty_ = false;
        openAsset(id);
        return;
    } else if (!confirm(this, tr("Change asset state"), tr("Apply the selected change to this saved asset?"))) {
        return;
    }
    mutate(action, params, [this, action](const auto &result, const auto &error) {
        if (error.isEmpty()) {
            setAsset(action == "asset.delete" ? QJsonObject{} : result);
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Asset updated."));
            refreshCatalog();
        }
    });
}

void GovernancePanel::showHistory()
{
    const auto expected = versioned();
    QDialog dialog(this);
    dialog.setWindowTitle(tr("Published revision history"));
    auto *layout = new QVBoxLayout(&dialog);
    auto *items = new QListWidget;
    items->setObjectName("governanceHistoryItems");
    auto *historyStatus = new QLabel;
    historyStatus->setWordWrap(true);
    auto *more = button(QT_TRANSLATE_NOOP("GovernancePanel", "Older revisions"), "governanceHistoryMore");
    auto *rollback = button(QT_TRANSLATE_NOOP("GovernancePanel", "Roll back selected revision"), "governanceRollback");
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close);
    buttons->addButton(more, QDialogButtonBox::ActionRole);
    buttons->addButton(rollback, QDialogButtonBox::ActionRole);
    layout->addWidget(items, 1);
    layout->addWidget(historyStatus);
    layout->addWidget(buttons);
    const auto cursor = std::make_shared<QJsonValue>();
    const QPointer<QDialog> dialogGuard(&dialog);
    const auto load = [this, expected, cursor, dialogGuard, items, historyStatus, more] {
        QJsonObject params{{"id", expected.value("id")}, {"limit", 40}};
        if (!cursor->isNull())
            params.insert("before", *cursor);
        more->setEnabled(false);
        send("governance.revision.history", params, [cursor, dialogGuard, items, historyStatus, more](const auto &result, const auto &error) {
            if (!dialogGuard)
                return;
            if (!error.isEmpty()) {
                historyStatus->setText(error);
                more->setEnabled(true);
                return;
            }
            for (const auto &value : result.value("items").toArray()) {
                const auto revision = value.toObject();
                auto *row = new QListWidgetItem(tr("r%1 · %2 · %3")
                    .arg(revision.value("revision").toInteger())
                    .arg(uiStatusLabel(revision.value("event").toString()), revision.value("digest").toString().left(16)), items);
                row->setData(Qt::UserRole, revision.value("revision").toVariant());
            }
            *cursor = result.value("next_before");
            more->setEnabled(cursor->isDouble());
        });
    };
    rollback->setEnabled(false);
    connect(items, &QListWidget::currentItemChanged, &dialog, [rollback](QListWidgetItem *item) {
        rollback->setEnabled(item != nullptr);
    });
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    connect(more, &QPushButton::clicked, &dialog, load);
    qint64 target = 0;
    connect(rollback, &QPushButton::clicked, &dialog, [&] {
        const auto revision = items->currentItem()->data(Qt::UserRole).toLongLong();
        if (confirm(&dialog, tr("Roll back published governance"),
                    tr("Activate the selected snapshot as a new revision? The revision history will be retained."))) {
            target = revision;
            dialog.accept();
        }
    });
    load();
    dialog.resize(560, 410);
    if (dialog.exec() != QDialog::Accepted || target <= 0 || expected != versioned())
        return;
    auto params = expected;
    params.insert("target_revision", target);
    mutate("change.rollback", params, [this](const auto &result, const auto &error) {
        if (error.isEmpty()) {
            setAsset(result);
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Rollback published as a new revision."));
            refreshCatalog();
        }
    });
}

void GovernancePanel::readImport(const std::function<void(const QString &, const QString &)> &accept)
{
    if (writing_)
        return;
    const auto path = QFileDialog::getOpenFileName(this, tr("Import UTF-8 text"), {},
        tr("Text resources (*.md *.yaml *.yml *.json *.txt);;All files (*)"));
    if (path.isEmpty())
        return;
    struct Input { QString name; QString text; QString error; };
    const auto input = std::make_shared<Input>();
    auto *worker = QThread::create([input, path] {
        constexpr qint64 limit = 240000;
        const QFileInfo info(path);
        QFile file(path);
        if (info.isSymLink() || !info.isFile() || !file.open(QIODevice::ReadOnly)) {
            input->error = tr("Choose a readable regular file without a symbolic link.");
            return;
        }
        const auto bytes = file.read(limit + 1);
        if (file.error() != QFileDevice::NoError || bytes.size() > limit || !file.atEnd()) {
            input->error = tr("The selected file could not be read within the import size limit.");
            return;
        }
        QStringDecoder decode(QStringDecoder::Utf8);
        input->text = decode(bytes);
        if (decode.hasError() || input->text.contains(QChar::Null))
            input->error = tr("The selected file is not UTF-8 text.");
        input->name = info.fileName();
    });
    writing_ = true;
    autosave_->stop();
    refreshControls();
    reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Reading the selected file…"));
    connect(worker, &QThread::finished, worker, &QObject::deleteLater);
    connect(worker, &QThread::finished, this, [this, input, accept] {
        writing_ = false;
        refreshControls();
        if (input->error.isEmpty()) {
            report({});
            accept(input->name, input->text);
        } else {
            report(input->error, true);
            if (dirty_ && online_)
                autosave_->start();
        }
    });
    worker->start();
}

void GovernancePanel::editAttachment(bool add)
{
    const auto row = fileList_->currentRow();
    if (!add && row < 0)
        return;
    const auto attachment = add ? QJsonObject{} : files_.at(row).toObject();
    QDialog dialog(this);
    dialog.setWindowTitle(add ? tr("Add resource file") : tr("Edit resource file"));
    auto *layout = new QVBoxLayout(&dialog);
    auto *path = field("governanceFilePath", QT_TRANSLATE_NOOP("GovernancePanel", "Asset-relative file path"));
    path->setMaxLength(240);
    path->setText(attachment.value("path").toString());
    auto *content = new GovernanceEditor;
    content->setObjectName("governanceFileContent");
    content->setPlainText(attachment.value("content").toString());
    auto *actions = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
    connect(actions, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(actions, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    layout->addWidget(path);
    layout->addWidget(content, 1);
    layout->addWidget(actions);
    dialog.resize(540, 380);
    if (dialog.exec() != QDialog::Accepted)
        return;
    const auto name = path->text().trimmed();
    QJsonObject value{{"path", name}, {"content", content->toPlainText()}};
    if (add) {
        files_.append(value);
        fileList_->addItem(name);
    } else {
        files_.replace(row, value);
        fileList_->item(row)->setText(name);
    }
    edited();
}


QWidget *GovernancePanel::makeAssignments()
{
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->addWidget(label(QT_TRANSLATE_NOOP("GovernancePanel", "Select a published asset in Library, then choose a repository binding here. A binding does not grant access."), "lead"));
    auto *actions = new QHBoxLayout;
    assignmentBinding_ = new QComboBox;
    assignmentBinding_->setObjectName("governanceAssignmentBinding");
    live::text(assignmentBinding_, [] { return tr("Repository binding"); }, &QWidget::setAccessibleName);
    actions->addWidget(assignmentBinding_, 1);
    assign_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Assign selected"), "governanceAssign", "primary");
    unassign_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Remove assignment"), "governanceUnassign");
    actions->addWidget(assign_);
    actions->addWidget(unassign_);
    layout->addLayout(actions);
    assignmentResults_ = readOnly("governanceAssignments");
    layout->addWidget(assignmentResults_, 1);
    connect(assignmentBinding_, &QComboBox::currentIndexChanged, this, [this] {
        refreshAssignments();
        refreshControls();
    });
    connect(assign_, &QPushButton::clicked, this, [this] { updateAssignment(true); });
    connect(unassign_, &QPushButton::clicked, this, [this] { updateAssignment(false); });
    return page;
}

QWidget *GovernancePanel::makeEffective()
{
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->addWidget(label(QT_TRANSLATE_NOOP("GovernancePanel", "Simulate the published context for a binding and an optional selected profile. This preview does not execute an agent or extend permissions."), "lead"));
    auto *filters = new QHBoxLayout;
    effectiveBinding_ = new QComboBox;
    effectiveBinding_->setObjectName("governanceEffectiveBinding");
    live::text(effectiveBinding_, [] { return tr("Repository binding"); }, &QWidget::setAccessibleName);
    effectiveMode_ = new QComboBox;
    for (const auto *mode : {"WORK", "VERIFY", "ACCEPT", "RELEASE", "AUDIT"})
        effectiveMode_->addItem(QString::fromLatin1(mode), QString::fromLatin1(mode));
    effectiveMode_->setAccessibleName(tr("Task mode"));
    auto *run = button(QT_TRANSLATE_NOOP("GovernancePanel", "Preview context"), "governancePreviewContext", "primary");
    filters->addWidget(effectiveBinding_, 1);
    filters->addWidget(effectiveMode_);
    filters->addWidget(run);
    layout->addLayout(filters);
    effectiveResults_ = readOnly("governanceEffective");
    layout->addWidget(effectiveResults_, 1);
    connect(effectiveBinding_, &QComboBox::currentIndexChanged, this, [this] {
        ++scopeGeneration_;
        effectiveResults_->clear();
    });
    connect(effectiveMode_, &QComboBox::currentIndexChanged, this, [this] {
        ++scopeGeneration_;
        effectiveResults_->clear();
    });
    connect(run, &QPushButton::clicked, this, &GovernancePanel::previewEffective);
    return page;
}

void GovernancePanel::repositoryCatalogChanged()
{
    if (!isVisible())
        return;
    if (pages_->currentIndex() == 1 || pages_->currentIndex() == 3) {
        ++scopeGeneration_;
        refreshScopes();
        if (pages_->currentIndex() == 1)
            refreshAssignments();
        refreshControls();
    }
}

void GovernancePanel::refreshScopes()
{
    const auto available = repositories_();
    for (auto *combo : {assignmentBinding_, effectiveBinding_}) {
        const QSignalBlocker block(combo);
        const auto selected = combo->currentData().toString();
        combo->clear();
        combo->addItem(tr("Choose repository binding"), QString{});
        for (const auto &value : available) {
            const auto item = value.toObject();
            const auto id = item.value("binding_id").toString();
            if (!id.isEmpty() && !item.value("archived").toBool())
                combo->addItem(item.value("display_name").toString(), id);
        }
        combo->setCurrentIndex(qMax(0, combo->findData(selected)));
    }
}

void GovernancePanel::refreshAssignments()
{
    const auto binding = assignmentBinding_->currentData().toString();
    const auto serial = ++scopeGeneration_;
    if (!online_ || binding.isEmpty()) {
        assignmentResults_->clear();
        return;
    }
    send("governance.assignment.list", {{"binding_id", binding}, {"limit", 100}},
         [this, serial](const auto &result, const auto &error) {
        if (serial != scopeGeneration_)
            return;
        if (!error.isEmpty()) {
            assignmentResults_->setPlainText(error);
            return;
        }
        QStringList lines;
        for (const auto &value : result.value("items").toArray()) {
            const auto item = value.toObject();
            lines.append(tr("%1 · %2 · r%3").arg(item.value("title").toString(), item.value("kind").toString())
                          .arg(item.value("active_revision").toInteger()));
        }
        if (lines.isEmpty())
            lines.append(tr("No published assets are assigned to this binding."));
        if (result.value("next_after").isString())
            lines.append(tr("Showing the first 100 assignments."));
        assignmentResults_->setPlainText(lines.join(u'\n'));
    });
}

void GovernancePanel::updateAssignment(bool enabled)
{
    const auto binding = assignmentBinding_->currentData().toString();
    if (asset_.isEmpty() || binding.isEmpty() || dirty_)
        return;
    auto params = versioned();
    params.insert("binding_id", binding);
    params.insert("enabled", enabled);
    mutate("assignment.update", params, [this](const auto &result, const auto &error) {
        if (error.isEmpty()) {
            setAsset(result);
            reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Repository assignment updated."));
            refreshAssignments();
        }
    });
}

void GovernancePanel::previewEffective()
{
    if (!online_) {
        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Connect the local runtime to preview context."), true);
        return;
    }
    QJsonObject params{{"mode", effectiveMode_->currentData().toString()}, {"budget_bytes", 32768}};
    const auto binding = effectiveBinding_->currentData().toString();
    if (!binding.isEmpty())
        params.insert("binding_id", binding);
    if (asset_.value("kind").toString() == "profile")
        params.insert("profile_id", asset_.value("id"));
    const auto serial = ++scopeGeneration_;
    send("governance.effective.preview", params, [this, serial](const auto &result, const auto &error) {
        if (serial != scopeGeneration_)
            return;
        if (!error.isEmpty()) {
            effectiveResults_->setPlainText(error);
            return;
        }
        QStringList lines{tr("English context · %1 of %2 bytes · %3 bytes deferred")
                              .arg(result.value("total_bytes").toInteger()).arg(result.value("budget_bytes").toInteger())
                              .arg(result.value("deferred_bytes").toInteger())};
        for (const auto *key : {"included", "excluded"}) {
            lines.append(QString{});
            lines.append(QString::fromLatin1(key) == "included" ? tr("Included and deferred") : tr("Excluded"));
            for (const auto &value : result.value(key).toArray()) {
                const auto item = value.toObject();
                lines.append(tr("%1 · r%2 · %3 · %4 · %5 bytes")
                    .arg(item.value("title").toString()).arg(item.value("revision").toInteger())
                    .arg(uiStatusLabel(item.value("source").toString()), uiStatusLabel(item.value("reason").toString()))
                    .arg(item.value("bytes").toInteger()));
            }
        }
        lines.append(QString{});
        lines.append(tr("Enforced restrictions remain unchanged:"));
        for (const auto &value : result.value("hard_denies").toArray())
            lines.append(uiStatusLabel(value.toString()));
        effectiveResults_->setPlainText(lines.join(u'\n'));
    });
}

QWidget *GovernancePanel::makeIntegrations()
{
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->addWidget(label(QT_TRANSLATE_NOOP("GovernancePanel", "Inspect pinned adapter capabilities and test an explicitly selected loopback server. Reachability does not prove authentication or execution."), "lead"));
    auto *filters = new QHBoxLayout;
    adapter_ = new QComboBox;
    adapter_->setObjectName("governanceAdapter");
    live::text(adapter_, [] { return tr("Adapter"); }, &QWidget::setAccessibleName);
    port_ = field("governanceAdapterPort", QT_TRANSLATE_NOOP("GovernancePanel", "Local server port"));
    port_->setValidator(new QIntValidator(1, 65535, port_));
    port_->setMaximumWidth(155);
    probe_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Probe loopback"), "governanceProbe", "primary");
    inspectImport_ = button(QT_TRANSLATE_NOOP("GovernancePanel", "Inspect import"), "governanceInspectImport");
    filters->addWidget(adapter_, 1);
    filters->addWidget(port_);
    filters->addWidget(probe_);
    filters->addWidget(inspectImport_);
    layout->addLayout(filters);
    adapterDetails_ = readOnly("governanceIntegrationDetails");
    layout->addWidget(adapterDetails_, 1);
    connect(probe_, &QPushButton::clicked, this, &GovernancePanel::probeIntegration);
    connect(inspectImport_, &QPushButton::clicked, this, &GovernancePanel::inspectImport);
    connect(adapter_, &QComboBox::currentIndexChanged, this, [this] {
        ++scopeGeneration_;
        const auto index = adapter_->currentIndex();
        if (index >= 0 && index < adapterItems_.size())
            adapterDetails_->setPlainText(pretty(adapterItems_.at(index)));
        refreshControls();
    });
    return page;
}

void GovernancePanel::loadIntegrations()
{
    if (!online_ || integrationLoading_)
        return;
    integrationLoading_ = true;
    refreshControls();
    send("integration.list", {}, [this](const auto &result, const auto &error) {
        integrationLoading_ = false;
        if (!error.isEmpty()) {
            adapterDetails_->setPlainText(error);
        } else {
            const auto selected = adapter_->currentData().toString();
            const QSignalBlocker block(adapter_);
            adapter_->clear();
            adapterItems_ = result.value("adapters").toArray();
            for (const auto &value : adapterItems_) {
                const auto item = value.toObject();
                adapter_->addItem(item.value("name").toString(), item.value("id").toString());
            }
            if (!adapterItems_.isEmpty()) {
                adapter_->setCurrentIndex(qMax(0, adapter_->findData(selected)));
                adapterDetails_->setPlainText(pretty(adapterItems_.at(adapter_->currentIndex())));
            }
        }
        refreshControls();
    });
}

void GovernancePanel::probeIntegration()
{
    if (!port_->hasAcceptableInput()) {
        port_->setFocus();
        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Enter a local server port from 1 to 65535."), true);
        return;
    }
    const auto id = adapter_->currentData().toString();
    const auto serial = scopeGeneration_;
    integrationLoading_ = true;
    refreshControls();
    send("integration.probe", {{"adapter", id}, {"port", port_->text().toInt()}}, [this, serial](const auto &result, const auto &error) {
        integrationLoading_ = false;
        refreshControls();
        if (serial == scopeGeneration_)
            adapterDetails_->setPlainText(error.isEmpty() ? pretty(result) : error);
    });
}

void GovernancePanel::inspectImport()
{
    const auto id = adapter_->currentData().toString();
    auto format = QStringLiteral("openhands-skill");
    if (id == "opencode") {
        bool accepted = false;
        const auto dialect = QInputDialog::getItem(this, tr("OpenCode import"), tr("Configuration dialect"),
                                                    {"V2", "V1"}, 0, false, &accepted);
        if (!accepted)
            return;
        format = dialect == "V1" ? "opencode-v1-agent" : "opencode-v2-agent";
    }
    const auto serial = scopeGeneration_;
    readImport([this, serial, format](const QString &, const QString &source) {
        if (serial != scopeGeneration_)
            return;
        send("integration.import_preview", {{"format", format}, {"source", source}},
             [this, serial](const auto &result, const auto &error) {
            if (serial == scopeGeneration_)
                adapterDetails_->setPlainText(error.isEmpty() ? pretty(result) : error);
        });
    });
}


void GovernancePanel::importBundle(const QString &source)
{
    const auto document = QJsonDocument::fromJson(source.toUtf8());
    if (!document.isObject()) {
        reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Choose a valid OPENRULES JSON bundle."), true);
        return;
    }
    const auto bundle = document.object();
    const auto serial = assetGeneration_;
    send("governance.asset.import.preview", {{"bundle", bundle}}, [this, serial, bundle](const auto &result, const auto &error) {
        if (serial != assetGeneration_)
            return;
        if (!error.isEmpty()) {
            report(error, true);
            return;
        }
        if (!abandonEdits())
            return;
        QDialog dialog(this);
        dialog.setWindowTitle(tr("Review bundle import"));
        auto *layout = new QVBoxLayout(&dialog);
        layout->addWidget(label(QT_TRANSLATE_NOOP("GovernancePanel", "Review origin, file paths, and dependencies. Import creates a draft.")));
        auto *details = readOnly("governanceBundlePreview");
        details->setPlainText(pretty(result));
        layout->addWidget(details, 1);
        auto *slug = field("governanceBundleSlug", QT_TRANSLATE_NOOP("GovernancePanel", "New slug if the original already exists"));
        slug->setMaxLength(100);
        layout->addWidget(slug);
        auto *actions = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
        connect(actions, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
        connect(actions, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        layout->addWidget(actions);
        dialog.resize(600, 440);
        if (dialog.exec() != QDialog::Accepted)
            return;
        QJsonObject params{{"bundle", bundle}, {"fingerprint", result.value("fingerprint")}};
        if (!slug->text().trimmed().isEmpty())
            params.insert("slug", slug->text().trimmed());
        mutate("asset.import.apply", params, [this](const auto &detail, const auto &failure) {
            if (failure.isEmpty()) {
                setAsset(detail);
                reportStatic(QT_TRANSLATE_NOOP("GovernancePanel", "Bundle imported as a draft."));
                refreshCatalog();
            }
        });
    });
}
