#include "main_window.h"
#include "build_info.h"
#include "notification_center.h"

#include <QAction>
#include <QApplication>
#include <QCloseEvent>
#include <QComboBox>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QMenu>
#include <QMessageBox>
#include <QPointer>
#include <QPushButton>
#include <QScreen>
#include <QShortcut>
#include <QSplitter>
#include <QStackedWidget>
#include <QTimer>
#include <QToolButton>
#include <QVBoxLayout>
#include <utility>

namespace {
QToolButton *iconButton(const QString &name, const QString &label)
{
    auto *button = new QToolButton;
    button->setObjectName(name);
    button->setAccessibleName(label);
    button->setToolTip(label);
    button->setIconSize(QSize(18, 18));
    button->setFocusPolicy(Qt::StrongFocus);
    return button;
}
}

MainWindow::MainWindow(RuntimeClient &client, QWidget *parent)
    : QMainWindow(parent), client_(client)
{
    setWindowTitle(QStringLiteral(CGACP_PRODUCT_GENERATION));
    setMinimumSize(620, 360);
    const QSize preferred(1366, 768);
    const auto *screen = QGuiApplication::primaryScreen();
    resize(screen ? preferred.boundedTo(screen->availableGeometry().size() - QSize(22, 40))
                            .expandedTo(minimumSize()) : preferred);
    theme_ = new ThemeManager(this);
    theme_->apply("dark", "comfortable", 100);

    preferencesTimer_ = new QTimer(this);
    preferencesTimer_->setSingleShot(true);
    preferencesTimer_->setInterval(320);
    connect(preferencesTimer_, &QTimer::timeout, this, &MainWindow::savePreferences);

    splitter_ = new QSplitter;
    splitter_->setObjectName("shellSplitter");
    splitter_->setChildrenCollapsible(false);
    splitter_->setHandleWidth(5);
    setCentralWidget(splitter_);

    sidebar_ = new QFrame;
    sidebar_->setObjectName("sidebar");
    auto *rail = new QVBoxLayout(sidebar_);
    rail->setContentsMargins(10, 12, 10, 9);
    rail->setSpacing(6);
    identityLayout_ = new QHBoxLayout;
    identityLayout_->setSpacing(5);
    auto *mark = new QLabel;
    mark->setAlignment(Qt::AlignCenter);
    mark->setObjectName("brandMark");
    mark->setAccessibleName(QStringLiteral("CIRNETLAB"));
    mark->setFixedSize(26, 26);
    mark->setPixmap(QApplication::windowIcon().pixmap(mark->size()));
    brand_ = new QLabel(QStringLiteral("CGACP"));
    brand_->setObjectName("brand");
    brandSubtitle_ = new QLabel(QStringLiteral("CIRNETLAB"));
    brandSubtitle_->setObjectName("brandSubtitle");
    sidebarToggle_ = iconButton("toggleSidebar", tr("Collapse sidebar"));
    identityLayout_->addWidget(mark, 0, Qt::AlignCenter);
    identityLayout_->addWidget(brand_, 0, Qt::AlignVCenter);
    identityLayout_->addWidget(brandSubtitle_, 1, Qt::AlignVCenter);
    identityLayout_->addWidget(sidebarToggle_, 0, Qt::AlignCenter);
    rail->addLayout(identityLayout_);
    rail->addSpacing(8);
    newChatAction_ = new QAction(tr("New Chat"), this);
    newChatAction_->setShortcut(QKeySequence::New);
    addAction(newChatAction_);
    chatButton_ = new QPushButton;
    chatButton_->setObjectName("navNewChat");
    chatButton_->setProperty("navigation", true);
    chatButton_->setAccessibleName(newChatAction_->text());
    chatButton_->setToolTip(newChatAction_->text());
    rail->addWidget(chatButton_);
    automationsButton_ = new QPushButton;
    automationsButton_->setObjectName("navAutomations");
    automationsButton_->setProperty("navigation", true);
    automationsButton_->setCheckable(true);
    automationsButton_->setAccessibleName(tr("Automations"));
    automationsButton_->setToolTip(tr("Automations"));
    rail->addWidget(automationsButton_);

    auto *content = new QWidget;
    content->setObjectName("content");
    content->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Expanding);
    auto *right = new QVBoxLayout(content);
    right->setContentsMargins(0, 0, 0, 0);
    pages_ = new QStackedWidget;
    pages_->setObjectName("pages");
    pages_->setMinimumWidth(0);
    right->addWidget(pages_);
    notifications_ = new NotificationCenter(*content);

    repositoriesPage_ = makeRepositories();
    conversationsPage_ = makeConversations();
    automationsPage_ = makeAutomations();
    settingsPage_ = makeSettings();
    for (auto *page : {conversationsPage_, automationsPage_, settingsPage_})
        pages_->addWidget(page);

    historyPage_ = makeConversationHistory();
    rail->addWidget(historyPage_, 1);
    auto *footer = new QFrame;
    footer->setObjectName("sidebarFooter");
    localLayout_ = new QHBoxLayout(footer);
    localLayout_->setContentsMargins(0, 0, 0, 0);
    localLayout_->setSpacing(4);
    localButton_ = iconButton("localMenu", tr("Local"));
    localButton_->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    localButton_->setProperty("navigation", true);
    auto *localMenu = new QMenu(localButton_);
    auto *runtimeMenu = localMenu->addAction(tr("Agents & Runtime"));
    connect(runtimeMenu, &QAction::triggered, this, [this] { showSettings("runtime"); });
    auto *historyMenu = localMenu->addAction(tr("Notifications"));
    connect(historyMenu, &QAction::triggered, this, [this] { showSettings("notifications"); });
    localButton_->setMenu(localMenu);
    localButton_->setPopupMode(QToolButton::InstantPopup);
    settingsButton_ = iconButton("navSettings", tr("Settings"));
    localLayout_->addWidget(localButton_, 0, Qt::AlignLeft);
    localLayout_->addStretch(1);
    localLayout_->addWidget(settingsButton_);
    rail->addWidget(footer);
    splitter_->addWidget(sidebar_);
    splitter_->addWidget(content);
    splitter_->setStretchFactor(0, 0);
    splitter_->setStretchFactor(1, 1);

    connect(theme_, &ThemeManager::changed, this, [this] {
        changeSidebar(sidebarCollapsed_);
        notifications_->refreshIcons();
    });
    changeSidebar(false);
    showPage(automationsPage_);
    setConnection(false, tr("Connecting to local runtime…"));
    savedPreferences_ = preferenceSettings();
    connect(splitter_, &QSplitter::splitterMoved, this, [this] {
        if (!sidebarCollapsed_) {
            sidebarWidth_ = qBound(210, splitter_->sizes().value(0, 260), 360);
            schedulePreferencesSave();
        }
    });
    connect(sidebarToggle_, &QToolButton::clicked, this, [this] {
        changeSidebar(!sidebarCollapsed_);
        schedulePreferencesSave();
    });
    connect(settingsButton_, &QToolButton::clicked, this, [this] { showSettings(); });
    connect(chatButton_, &QPushButton::clicked, newChatAction_, &QAction::trigger);
    connect(newChatAction_, &QAction::triggered, this, [this] {
        showPage(conversationsPage_);
        newConversation();
    });
    connect(automationsButton_, &QPushButton::clicked, this, [this] { showPage(automationsPage_); });

    auto *openSettings = new QShortcut(QKeySequence(QStringLiteral("Ctrl+,")), this);
    connect(openSettings, &QShortcut::activated, this, [this] { showSettings(); });
    auto *sidebarShortcut = new QShortcut(QKeySequence(QStringLiteral("Ctrl+B")), this);
    connect(sidebarShortcut, &QShortcut::activated, sidebarToggle_, &QToolButton::click);
    auto *find = new QShortcut(QKeySequence::Find, this);
    connect(find, &QShortcut::activated, this, [this] {
        QLineEdit *search = nullptr;
        if (pages_->currentWidget() == settingsPage_ && settingsChoice_->currentData().toString() == "repositories")
            search = repoSearch_;
        else if (pages_->currentWidget() == automationsPage_)
            search = automationSearch_;
        else if (pages_->currentWidget() == conversationsPage_) {
            if (sidebarCollapsed_) {
                changeSidebar(false);
                schedulePreferencesSave();
            }
            search = conversationSearch_;
        }
        if (search) {
            search->setFocus(Qt::ShortcutFocusReason);
            search->selectAll();
        }
    });
    auto *back = new QShortcut(QKeySequence(Qt::Key_Escape), settingsPage_);
    back->setContext(Qt::WidgetWithChildrenShortcut);
    connect(back, &QShortcut::activated, this, &MainWindow::closeSettings);

    connect(&client_, &RuntimeClient::ready, this, [this] {
        connecting_ = true;
        setConnection(false, tr("Connecting to local runtime…"));
        call("runtime.info", {}, [this](const QJsonObject &info, const QString &error) {
            connecting_ = false;
            if (!error.isEmpty() || info.value("version").toString() != QStringLiteral(CGACP_APP_VERSION)) {
                const auto reason = error.isEmpty() ? tr("Dashboard and runtime versions differ.") : error;
                setConnection(false, reason);
                showStatus(reason, true);
                client_.stop();
                return;
            }
            runtimeInfo_ = info;
            setConnection(true, tr("Local runtime connected"));
            loadPreferences();
            refreshRepositories();
            refreshConversations();
            refreshAutomations();
            if (!currentConversation_.isEmpty())
                openConversation(currentConversation_);
        });
    });
    connect(&client_, &RuntimeClient::disconnected, this, [this](const QString &reason) {
        const bool changed = online_ || connecting_ || connectionStatus_ != reason;
        connecting_ = false;
        setConnection(false, reason);
        if (changed && !shutdownPending_)
            showStatus(reason, true);
    });
    connect(&client_, &RuntimeClient::stopped, this, [this] {
        if (shutdownPending_)
            QCoreApplication::quit();
        else if (!online_)
            updateRuntimeDetails();
    });
    poll_ = new QTimer(this);
    poll_->setInterval(4000);
    connect(poll_, &QTimer::timeout, this, &MainWindow::refreshConversationEvents);
    poll_->start();
}

void MainWindow::setConnection(bool ready, const QString &status)
{
    online_ = ready;
    connectionStatus_ = status;
    updateShellIcons();
    updateRuntimeDetails();
    updateConversationAvailability();
    updateAutomationAvailability();
    updateRepositoryActions();
}

void MainWindow::reconnectRuntime()
{
    if (online_ || client_.isRunning())
        return;
    connecting_ = true;
    setConnection(false, tr("Connecting to local runtime…"));
    client_.start();
}

void MainWindow::changeSidebar(bool collapsed)
{
    sidebarCollapsed_ = collapsed;
    const int brandWidth = 26 + brand_->sizeHint().width() + brandSubtitle_->sizeHint().width() +
                           sidebarToggle_->sizeHint().width() + 45;
    const int minimum = qBound(210, brandWidth, 360);
    sidebar_->setMinimumWidth(0);
    sidebar_->setMaximumWidth(collapsed ? 60 : 360);
    brand_->setVisible(!collapsed);
    brandSubtitle_->setVisible(!collapsed);
    historyPage_->setVisible(!collapsed);
    identityLayout_->setDirection(collapsed ? QBoxLayout::TopToBottom : QBoxLayout::LeftToRight);
    localLayout_->setDirection(collapsed ? QBoxLayout::TopToBottom : QBoxLayout::LeftToRight);
    sidebar_->setMinimumWidth(collapsed ? 60 : minimum);
    const int next = collapsed ? 60 : qMax(minimum, sidebarWidth_);
    splitter_->setSizes({next, qMax(1, splitter_->width() - next - splitter_->handleWidth())});
    if (auto *handle = splitter_->handle(1))
        handle->setEnabled(!collapsed);
    updateShellIcons();
}

void MainWindow::updateShellIcons()
{
    if (!sidebarToggle_ || !localButton_)
        return;
    const auto toggle = sidebarCollapsed_ ? tr("Expand sidebar") : tr("Collapse sidebar");
    sidebarToggle_->setToolTip(toggle);
    sidebarToggle_->setAccessibleName(toggle);
    sidebarToggle_->setIcon(uiIcon(sidebarCollapsed_ ? UiIcon::Expand : UiIcon::Collapse));
    settingsButton_->setIcon(uiIcon(UiIcon::Settings));
    chatButton_->setText(sidebarCollapsed_ ? QString{} : newChatAction_->text());
    chatButton_->setIcon(uiIcon(UiIcon::Add));
    automationsButton_->setText(sidebarCollapsed_ ? QString{} : tr("Automations"));
    automationsButton_->setIcon(uiIcon(UiIcon::Automations));
    localButton_->setText(sidebarCollapsed_ ? QString{} : tr("Local"));
    localButton_->setToolButtonStyle(sidebarCollapsed_ ? Qt::ToolButtonIconOnly : Qt::ToolButtonTextBesideIcon);
    localButton_->setToolTip(connectionStatus_);
    localButton_->setAccessibleName(tr("Local — %1").arg(connectionStatus_));
    const bool light = QApplication::palette().color(QPalette::Window).lightness() > 128;
    const QColor status = online_ ? QColor(light ? "#086a55" : "#75d8a4") :
                         connecting_ ? QColor(light ? "#7e540a" : "#e9bf7f") :
                                       QColor(light ? "#a32f27" : "#ffaaa3");
    localButton_->setIcon(uiIcon(UiIcon::Status, status));
}

void MainWindow::showPage(QWidget *page)
{
    pages_->setCurrentWidget(page);
    automationsButton_->setChecked(page == automationsPage_);
}

void MainWindow::call(const QString &method, QJsonObject params, RuntimeClient::Reply reply)
{
    const QPointer<MainWindow> owner(this);
    client_.request(method, std::move(params), [owner, reply = std::move(reply)](
                                               const QJsonObject &result, const QString &error) {
        if (owner && reply)
            reply(result, error);
    });
}

void MainWindow::showStatus(const QString &text, bool error)
{
    notifications_->post(text, error ? QStringLiteral("error") : QStringLiteral("success"));
}

void MainWindow::showError(const QString &text)
{
    showStatus(text, true);
}

void MainWindow::closeEvent(QCloseEvent *event)
{
    if (shutdownPending_ || closeAfterSave_) {
        event->ignore();
        return;
    }
    if (!client_.isRunning() && !preferencesDirty_) {
        event->accept();
        return;
    }
    const bool discard = preferencesDirty_ && (!online_ || preferencesRetryRequired_ || !preferencesLoaded_);
    const auto answer = QMessageBox::question(
        this, tr("Close CGACP"),
        discard ? tr("Some preferences are unsaved. Close CGACP without saving them?") :
                  tr("Stop the local runtime and close CGACP? Saved conversations and runs can be restored on reopen."),
        QMessageBox::Yes | QMessageBox::Cancel, QMessageBox::Cancel);
    if (answer == QMessageBox::Cancel) {
        event->ignore();
        return;
    }
    if (!client_.isRunning()) {
        event->accept();
        return;
    }
    event->ignore();
    if (preferencesDirty_ && !discard) {
        closeAfterSave_ = true;
        savePreferences();
    } else {
        stopRuntimeAndClose();
    }
}

void MainWindow::stopRuntimeAndClose()
{
    shutdownPending_ = true;
    hide();
    if (client_.isRunning())
        client_.stop();
    else
        QCoreApplication::quit();
}
