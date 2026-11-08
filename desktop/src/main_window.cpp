#include "main_window.h"
#include "build_info.h"

#include <QApplication>
#include <QButtonGroup>
#include <QCloseEvent>
#include <QFile>
#include <QFrame>
#include <QHBoxLayout>
#include <QIcon>
#include <QJsonDocument>
#include <QLabel>
#include <QMessageBox>
#include <QPointer>
#include <QPushButton>
#include <QResizeEvent>
#include <QScreen>
#include <QStackedWidget>
#include <QTimer>
#include <QVBoxLayout>
#include <utility>

void applyTheme(QApplication &app)
{
    Q_INIT_RESOURCE(cgacp_ui);
    app.setWindowIcon(QIcon(QStringLiteral(":/cgacp/favicon.png")));
    app.setStyle(QStringLiteral("Fusion"));
    QFile qss(QStringLiteral(":/cgacp/dark.qss"));
    if (qss.open(QIODevice::ReadOnly))
        app.setStyleSheet(QString::fromUtf8(qss.readAll()));
}

MainWindow::MainWindow(RuntimeClient &client, QWidget *parent)
    : QMainWindow(parent), client_(client)
{
    setWindowTitle(QStringLiteral(CGACP_PRODUCT_GENERATION));
    setMinimumSize(860, 460);
    const QSize preferred(1366, 768);
    const auto *screen = QGuiApplication::primaryScreen();
    resize(screen ? preferred.boundedTo(screen->availableGeometry().size() - QSize(22, 40))
                            .expandedTo(minimumSize())
                  : preferred);

    auto *shell = new QWidget;
    auto *shellLayout = new QHBoxLayout(shell);
    shellLayout->setSpacing(0);
    shellLayout->setContentsMargins(0, 0, 0, 0);
    setCentralWidget(shell);
    auto *sidebar = new QFrame;
    sidebar->setObjectName("sidebar");
    sidebar->setFixedWidth(230);
    auto *rail = new QVBoxLayout(sidebar);
    rail->setContentsMargins(17, 23, 17, 16);
    rail->setSpacing(8);
    auto *identity = new QHBoxLayout;
    auto *mark = new QLabel;
    mark->setAlignment(Qt::AlignCenter);
    mark->setObjectName("brandMark");
    mark->setFixedSize(35, 35);
    mark->setPixmap(QApplication::windowIcon().pixmap(mark->size()));
    identity->addWidget(mark);
    auto *brandText = new QVBoxLayout;
    brandText->setSpacing(0);
    auto *brand = new QLabel(QStringLiteral("CGACP"));
    brand->setObjectName("brand");
    brandText->addWidget(brand);
    auto *subtitle = new QLabel(QStringLiteral("CIRNETLAB"));
    subtitle->setObjectName("eyebrow");
    brandText->addWidget(subtitle);
    identity->addLayout(brandText, 1);
    rail->addLayout(identity);
    rail->addSpacing(26);
    auto *workspace = new QLabel(tr("WORKSPACE"));
    workspace->setObjectName("eyebrow");
    rail->addWidget(workspace);
    auto *chat = new QPushButton(tr("+  New Chat"));
    chat->setObjectName("navNewChat");
    chat->setProperty("primary", true);
    chat->setAccessibleName(tr("New Chat"));
    rail->addWidget(chat);
    rail->addSpacing(8);

    pages_ = new QStackedWidget;
    repositoriesPage_ = makeRepositories();
    conversationsPage_ = makeConversations();
    automationsPage_ = makeAutomations();
    pages_->addWidget(repositoriesPage_);
    pages_->addWidget(conversationsPage_);
    pages_->addWidget(automationsPage_);
    auto *group = new QButtonGroup(this);
    auto nav = [this, rail, group](const QString &label, const QString &name, QWidget *page) {
        auto *button = new QPushButton(label);
        button->setObjectName(name);
        button->setProperty("navigation", true);
        button->setCheckable(true);
        group->addButton(button);
        rail->addWidget(button);
        connect(button, &QPushButton::clicked, this, [this, page] { showPage(page); });
        return button;
    };
    nav(tr("Automations"), "navAutomations", automationsPage_)->setChecked(true);
    nav(tr("Conversations"), "navConversations", conversationsPage_);
    nav(tr("Repositories"), "navRepositories", repositoriesPage_);
    rail->addStretch();

    connection_ = new QLabel(tr("Offline"));
    connection_->setObjectName("connection");
    connection_->setTextFormat(Qt::PlainText);
    rail->addWidget(connection_);
    reconnect_ = new QPushButton(tr("Reconnect"));
    reconnect_->setAccessibleName(tr("Reconnect"));
    rail->addWidget(reconnect_);
    auto *aboutButton = new QPushButton(tr("About CGACP"));
    aboutButton->setProperty("navigation", true);
    rail->addWidget(aboutButton);
    shellLayout->addWidget(sidebar);

    auto *content = new QWidget;
    content->setObjectName("content");
    auto *right = new QVBoxLayout(content);
    right->setContentsMargins(0, 0, 0, 0);
    right->setSpacing(0);
    right->addWidget(pages_, 1);
    feedback_ = new QLabel(tr("Select a repository to get started."));
    feedback_->setObjectName("feedback");
    feedback_->setTextFormat(Qt::PlainText);
    feedback_->setWordWrap(true);
    right->addWidget(feedback_);
    shellLayout->addWidget(content, 1);
    showPage(automationsPage_);
    setConnection(false, tr("Connecting to local runtime…"));

    connect(chat, &QPushButton::clicked, this, [this] {
        showPage(conversationsPage_);
        newConversation();
    });
    connect(aboutButton, &QPushButton::clicked, this, &MainWindow::about);
    connect(reconnect_, &QPushButton::clicked, this, [this] {
        setConnection(false, tr("Connecting to local runtime…"));
        client_.start();
    });
    connect(&client_, &RuntimeClient::ready, this, [this] {
        call(QStringLiteral("runtime.info"), {}, [this](const QJsonObject &info, const QString &error) {
            if (!error.isEmpty()) {
                setConnection(false, error);
                return;
            }
            if (info.value("version").toString() != QStringLiteral(CGACP_APP_VERSION)) {
                setConnection(false, tr("Dashboard and runtime versions differ."));
                return;
            }
            runtimeInfo_ = info;
            setConnection(true, tr("Local runtime connected"));
            refreshRepositories();
            refreshConversations();
            refreshTemplates();
            refreshAutomations();
            if (!currentConversation_.isEmpty())
                openConversation(currentConversation_);
        });
    });
    connect(&client_, &RuntimeClient::disconnected, this, [this](const QString &reason) {
        setConnection(false, reason);
        showStatus(reason, true);
    });
    connect(&client_, &RuntimeClient::stopped, this, [this] {
        if (shutdownPending_)
            QCoreApplication::quit();
    });
    poll_ = new QTimer(this);
    poll_->setInterval(4000);
    connect(poll_, &QTimer::timeout, this, &MainWindow::refreshConversationEvents);
    poll_->start();
}

void MainWindow::setConnection(bool ready, const QString &status)
{
    online_ = ready;
    connection_->setText(ready ? tr("●  Local runtime connected") : tr("○  Runtime offline"));
    connection_->setToolTip(status);
    reconnect_->setVisible(!ready);
    reconnect_->setEnabled(!client_.isRunning());
    if (!ready) {
        QTimer::singleShot(200, this, [this] {
            if (!online_)
                reconnect_->setEnabled(!client_.isRunning());
        });
    }
    showStatus(status, !ready);
}

void MainWindow::showPage(QWidget *page)
{
    pages_->setCurrentWidget(page);
    const auto navigationName = page == repositoriesPage_ ? "navRepositories"
                              : page == conversationsPage_ ? "navConversations" : "navAutomations";
    if (auto *navigation = findChild<QPushButton *>(QString::fromLatin1(navigationName)))
        navigation->setChecked(true);
    if (page == repositoriesPage_ && online_)
        refreshRepositories();
    if (page == conversationsPage_ && online_)
        refreshConversations();
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
    if (!feedback_)
        return;
    feedback_->setProperty("error", error);
    feedback_->setText(text);
    feedback_->setStyleSheet(error ? QStringLiteral("color: #ff9d95") : QString());
}

void MainWindow::showError(const QString &text)
{
    showStatus(text, true);
    QMessageBox dialog(QMessageBox::Warning, tr("CGACP"), text, QMessageBox::Ok, this);
    dialog.setTextFormat(Qt::PlainText);
    dialog.exec();
}

void MainWindow::about()
{
    const auto release = runtimeInfo_.value("release_name").toString(QStringLiteral(CGACP_RELEASE_NAME));
    const auto native = runtimeInfo_.value("native_agent").toObject();
    QStringList adapters;
    for (auto it = native.begin(); it != native.end(); ++it)
        adapters.append(it.key() + QStringLiteral(" · ") + it.value().toString());
    QMessageBox dialog(QMessageBox::Information, tr("About CGACP"),
                       tr("%1\nApplication %2\nRelease %3\n\nNative execution capability: %4")
                           .arg(QStringLiteral(CGACP_PRODUCT_GENERATION), QStringLiteral(CGACP_APP_VERSION),
                                release, adapters.isEmpty() ? tr("Unavailable") : adapters.join(QStringLiteral("\n"))),
                       QMessageBox::Ok, this);
    dialog.setTextFormat(Qt::PlainText);
    dialog.exec();
}

void MainWindow::closeEvent(QCloseEvent *event)
{
    if (shutdownPending_) {
        event->ignore();
        return;
    }
    if (!client_.isRunning()) {
        event->accept();
        return;
    }
    const auto answer = QMessageBox::question(
        this, tr("Stop local runtime"),
        tr("Stop the local runtime and close CGACP? Saved conversations and runs can be restored on reopen."),
        QMessageBox::Yes | QMessageBox::Cancel, QMessageBox::Cancel);
    if (answer == QMessageBox::Cancel) {
        event->ignore();
        return;
    }
    shutdownPending_ = true;
    event->ignore();
    hide();
    client_.stop();
}

void MainWindow::resizeEvent(QResizeEvent *event)
{
    QMainWindow::resizeEvent(event);
    if (automationCards_) {
        const auto columns = width() >= 1120 ? 2 : 1;
        if (columns != cardColumns_)
            QTimer::singleShot(0, this, &MainWindow::renderAutomationCards);
    }
}
