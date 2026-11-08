#include "build_info.h"
#include "main_window.h"
#include "runtime_client.h"
#include "notification_center.h"
#include "pages_test.h"
#include "repository_cases.h"

#include <QAbstractItemModel>
#include <QAction>
#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QCoreApplication>
#include <QDateTime>
#include <QDialogButtonBox>
#include <QDir>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QFont>
#include <QFontDatabase>
#include <QIcon>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMenu>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollArea>
#include <QScrollBar>
#include <QSpinBox>
#include <QStyleHints>
#include <QSignalSpy>
#include <QSplitter>
#include <QStandardPaths>
#include <QStyleOption>
#include <QTest>
#include <QTemporaryDir>
#include <QTableView>
#include <QTimer>
#include <QToolButton>
#include <QTranslator>
#include <algorithm>
#include <cstdio>

namespace {
QJsonObject fixtureResult(const QString &method, const QJsonObject &params,
                          const QJsonArray &methods, const QString &mode)
{
    if (method == "runtime.info")
        return {{"product_generation", QStringLiteral(CGACP_PRODUCT_GENERATION)},
                {"version", QStringLiteral(CGACP_APP_VERSION)},
                {"release_name", QStringLiteral(CGACP_RELEASE_NAME)},
                {"native_agent", QJsonObject{{"codex", "HANDOFF_ONLY"}}},
                {"execution_enforcement", QJsonObject{{"ISOLATED_EXECUTION", "BLOCKED"}}}};
    if (method == "preferences.get")
        return {{"revision", 0}, {"settings", QJsonObject{{"theme", "dark"}, {"density", "comfortable"},
                {"text_scale", 100}, {"sidebar_width", 260}, {"sidebar_collapsed", false},
                {"repository_layout", QJsonValue(QJsonValue::Null)}, {"notification_level", "all"}}}};
    if (method == "preferences.set")
        return {{"revision", params.value("expected_revision").toInteger() + 1}, {"settings", params.value("settings")}};
    if (method == "test.methods")
        return {{"items", methods}};
    if (mode == "paged" && method == "repo.list") {
        const bool second = params.value("after") == "fixture-page-2";
        QJsonArray items;
        const auto row = [](int number, const QString &name) {
            return QJsonObject{{"display_name", name}, {"root", "/fixture/" + name}, {"branch", "main"},
                               {"binding_id", QString("binding-%1").arg(number, 32, 16, QChar('0'))},
                               {"repo_id", QString("repo-%1").arg(number, 32, 16, QChar('0'))},
                               {"status", "ACTIVE"}, {"revision", 1}, {"repo_revision", 1}};
        };
        if (second) {
            items.append(row(41, "Last page marker"));
            return {{"items", items}};
        }
        for (int number = 1; number <= 40; ++number)
            items.append(row(number, QString("Workspace %1").arg(number)));
        return {{"items", items}, {"next", "fixture-page-2"}};
    }
    if (method == "repo.select")
        return {{"binding_id", params.value("binding_id")}, {"status", "ACTIVE"}};
    if (method == "repo.list")
        return {{"items", QJsonArray{QJsonObject{{"display_name", "Fixture UI"},
                 {"root", R"(\\?\C:\Fixtures\Workspaces\Unicode and spaces\2026\Dashboard UX verification\Fixture UI)"},
                 {"repo_id", "repo-fixture"}, {"branch", "fixture-branch"},
                 {"binding_id", "binding-11111111111111111111111111111111"}, {"status", "ACTIVE"}, {"revision", 1}}}}};
    if (mode == "paged-conversations" && method == "conversation.list") {
        const auto cursor = params.value("before").toString().toInt();
        const int first = cursor > 0 ? cursor + 1 : 1;
        const int last = params.value("query").toString().isEmpty() ? qMin(first + 29, 65) : first;
        QJsonArray items;
        for (int index = first; index <= last; ++index) {
            items.append(QJsonObject{{"id", QString::number(index)}, {"title", QString("Fixture chat %1").arg(index)},
                                     {"state", "ACTIVE"}, {"revision", 1}});
        }
        return {{"items", items}, {"next_before", items.isEmpty() ? QJsonValue() : QJsonValue(QString::number(last))}};
    }
    if (method == "conversation.list") {
        const auto now = QDateTime::currentSecsSinceEpoch();
        return {{"items", QJsonArray{QJsonObject{{"id", "chat-1"}, {"title", "Fixture chat"}, {"state", "ACTIVE"},
                                               {"updated_at", now - 90}, {"revision", 1}},
                                      QJsonObject{{"id", "chat-2"}, {"title", "Other chat"}, {"state", "ACTIVE"},
                                                  {"updated_at", now - 3600}, {"revision", 1}}}}};
    }
    if (method == "conversation.get")
        return {{"conversation", QJsonObject{{"id", params.value("id")}, {"title", params.value("id") == "chat-1" ?
                             "Fixture chat" : "Other chat"}, {"state", "ACTIVE"}, {"revision", 1}}},
                {"messages", QJsonArray{QJsonObject{{"role", "USER"}, {"content", "Saved fixture message"}}}}};
    if (method == "automation.list")
        return {{"items", mode == "empty" ? QJsonArray{} : QJsonArray{
                    QJsonObject{{"id", "read-check"}, {"name", "Scoped review"}, {"prompt", "Review the selected worktree."},
                                {"binding_id", "binding-11111111111111111111111111111111"}, {"enabled", true}, {"revision", 1},
                                {"trigger", QJsonObject{{"kind", "DAILY"}, {"hour", 9}, {"minute", 0}, {"timezone", "UTC"}}}},
                    QJsonObject{{"id", "check-docs"}, {"name", "Documentation review"}, {"prompt", "Verify links in the local documentation."},
                                {"binding_id", "binding-11111111111111111111111111111111"}, {"enabled", true}, {"revision", 1},
                                {"trigger", QJsonObject{{"kind", "DAILY"}, {"hour", 14}, {"minute", 30}, {"timezone", "UTC"}}}},
                    QJsonObject{{"id", "inactive-check"}, {"name", "Review later"}, {"prompt", "Inspect local files."},
                                {"enabled", false}, {"revision", 1}}
                }}};
    return {{"items", QJsonArray{}}};
}

int runFixture(const QString &mode)
{
    char line[65536];
    QJsonArray methods;
    QJsonObject lastRepoSelection;
    while (std::fgets(line, sizeof line, stdin)) {
        const auto request = QJsonDocument::fromJson(QByteArray(line)).object();
        const auto method = request.value("method").toString();
        if (method == "test.invalid") {
            std::fputs("{malformed}\n", stdout);
            std::fflush(stdout);
            continue;
        }
        if (method == "test.crash")
            return 18;
        if (method == "repo.select")
            lastRepoSelection = request.value("params").toObject();
        methods.append(method);
        QJsonObject responseObject{{"v", 1}, {"id", request.value("id")}};
        if (mode == "save-error" && method == "preferences.set") {
            responseObject.insert("ok", false);
            responseObject.insert("error", QJsonObject{{"code", "STORAGE_UNAVAILABLE"}, {"message", "Fixture write denied."}});
        } else {
            responseObject.insert("ok", true);
            responseObject.insert("result", method == "test.lastSelection" ? lastRepoSelection :
                                  fixtureResult(method, request.value("params").toObject(), methods, mode));
        }
        const auto response = QJsonDocument(responseObject).toJson(QJsonDocument::Compact) + '\n';
        const auto half = response.size() / 2;
        std::fwrite(response.data(), 1, size_t(half), stdout);
        std::fflush(stdout);
        QTest::qSleep(7);
        std::fwrite(response.data() + half, 1, size_t(response.size() - half), stdout);
        std::fflush(stdout);
    }
    return 0;
}

QJsonObject invoke(RuntimeClient &client, const QString &method, QJsonObject params = {})
{
    QEventLoop loop;
    QJsonObject result;
    QString error = "The test request did not finish.";
    client.request(method, std::move(params), [&](const QJsonObject &value, const QString &failure) {
        result = value;
        error = failure;
        loop.quit();
    });
    if (error == "The test request did not finish.")
        loop.exec();
    if (!error.isEmpty())
        qWarning().noquote() << method << error;
    return result;
}

bool runGit(const QString &git, const QString &workingDirectory, const QStringList &args)
{
    QProcess process;
    process.setWorkingDirectory(workingDirectory);
    process.start(git, args);
    if (!process.waitForStarted(3000) || !process.waitForFinished(5000) ||
        process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0) {
        qWarning().noquote() << QString::fromUtf8(process.readAllStandardError());
        return false;
    }
    return true;
}

void settingsSection(MainWindow &window, const QString &section)
{
    auto *settings = window.findChild<QToolButton *>("navSettings");
    QVERIFY(settings);
    QTest::mouseClick(settings, Qt::LeftButton);
    auto *choice = window.findChild<QComboBox *>("settingsSection");
    QVERIFY(choice && choice->isVisible());
    const auto index = choice->findData(section);
    QVERIFY(index >= 0);
    choice->setCurrentIndex(index);
}

bool captureWidget(QWidget &widget, const QString &name)
{
    const auto directory = qEnvironmentVariable("CGACP_UI_CAPTURE");
    if (directory.isEmpty())
        return true;
    QTest::qWait(60);
    return widget.grab().save(directory + "/" + name + ".png");
}
}

class DesktopTest final : public QObject {
    Q_OBJECT
private slots:
    void fragmentedResponsesAndRestart()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        QSignalSpy connected(&client, &RuntimeClient::ready);
        QSignalSpy disconnected(&client, &RuntimeClient::disconnected);
        client.start();
        QTRY_COMPARE(connected.size(), 1);
        QJsonObject result;
        client.request("repo.list", {}, [&](const QJsonObject &value, const QString &error) {
            QVERIFY(error.isEmpty());
            result = value;
        });
        QTRY_COMPARE(result.value("items").toArray().size(), 1);
        client.request("test.invalid", {}, {});
        QTRY_COMPARE(disconnected.size(), 1);
        QTRY_VERIFY(!client.isRunning());
        client.start();
        QTRY_COMPARE(connected.size(), 2);
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }

    void responsiveNavigationAndCapture()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        MainWindow window(client);
        window.resize(1366, 768);
        window.show();
        QElapsedTimer startup;
        startup.start();
        client.start();
        auto *mark = window.findChild<QLabel *>("brandMark");
        auto *brand = window.findChild<QLabel *>("brand");
        auto *subtitle = window.findChild<QLabel *>("brandSubtitle");
        auto *split = window.findChild<QSplitter *>("shellSplitter");
        auto *sidebar = window.findChild<QWidget *>("sidebar");
        auto *toggle = window.findChild<QToolButton *>("toggleSidebar");
        auto *automations = window.findChild<QPushButton *>("navAutomations");
        QVERIFY(mark && brand && subtitle && split && sidebar && toggle && automations);
        QVERIFY(!mark->pixmap().isNull());
        const auto y = [sidebar](QWidget *widget) { return widget->mapTo(sidebar, widget->rect().center()).y(); };
        QVERIFY(qAbs(y(mark) - y(brand)) <= 2);
        QVERIFY(qAbs(y(brand) - y(subtitle)) <= 2);
        QVERIFY(mark->geometry().right() < brand->geometry().left());
        QVERIFY(brand->geometry().right() < subtitle->geometry().left());
        QVERIFY(!window.findChild<QWidget *>("feedback"));
        QVERIFY(!window.findChild<QWidget *>("navRepositories"));
        auto *footer = window.findChild<QWidget *>("sidebarFooter");
        QCOMPARE(footer->findChildren<QToolButton *>().size(), 2);
        QCOMPARE(footer->width(), sidebar->width());
        QCOMPARE(window.findChild<QToolButton *>("localMenu")->text(), QString("Local"));
        QVERIFY(captureWidget(*footer, "sidebar-footer"));
        QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 3);
        const auto initialMethods = invoke(client, "test.methods").value("items").toArray();
        qInfo() << "Populated fixture ready ms" << startup.elapsed() << "requests through readiness" << initialMethods.size() - 1;
        const auto ratio = window.devicePixelRatioF();
        qInfo() << "Qt platform" << QGuiApplication::platformName() << "devicePixelRatio" << ratio;
        for (const auto size : {QSize(1366, 768), QSize(1920, 1080)}) {
            window.resize(qRound(size.width() / ratio), qRound(size.height() / ratio));
            QTest::qWait(60);
            const auto shot = window.grab();
            qInfo() << "Capture request" << size << "logical" << window.size() << "actual pixels" << shot.size();
            QVERIFY(qAbs(shot.width() - size.width()) <= 2);
            QVERIFY(qAbs(shot.height() - size.height()) <= 2);
            QVERIFY(captureWidget(window, QString("automations-populated-%1x%2").arg(size.width()).arg(size.height())));
        }
        window.resize(1366, 768);

        const auto expanded = split->sizes().first();
        const auto initialRequests = invoke(client, "test.methods").value("items").toArray();
        auto *handle = split->handle(1);
        const auto center = handle->rect().center();
        QTest::mousePress(handle, Qt::LeftButton, Qt::NoModifier, center);
        QTest::mouseMove(handle, center + QPoint(46, 0), 30);
        QTest::mouseRelease(handle, Qt::LeftButton, Qt::NoModifier, center);
        QTRY_VERIFY(split->sizes().first() > expanded);
        const auto dragged = split->sizes().first();
        QTest::mouseClick(toggle, Qt::LeftButton);
        QCOMPARE(split->sizes().first(), 60);
        QVERIFY(!brand->isVisible());
        QVERIFY(window.findChild<QToolButton *>("navSettings")->isVisible());
        QVERIFY(captureWidget(window, "sidebar-collapsed"));
        QTest::mouseClick(toggle, Qt::LeftButton);
        QCOMPARE(split->sizes().first(), dragged);
        const auto resizedRequests = invoke(client, "test.methods").value("items").toArray();
        for (qsizetype index = initialRequests.size(); index < resizedRequests.size(); ++index) {
            const auto method = resizedRequests.at(index).toString();
            QVERIFY(method == "preferences.set" || method == "test.methods");
        }
        settingsSection(window, "appearance");
        QVERIFY(captureWidget(window, "settings-appearance"));
        auto *themes = window.findChild<QComboBox *>("themeChoice");
        auto *density = window.findChild<QComboBox *>("densityChoice");
        QVERIFY(themes && density);
        themes->setCurrentIndex(themes->findData("light"));
        QVERIFY(QApplication::palette().color(QPalette::Window).lightness() > 128);
        density->setCurrentIndex(density->findData("compact"));
        QVERIFY(captureWidget(window, "settings-appearance-light"));

        settingsSection(window, "repositories");
        auto *checkbox = window.findChild<QCheckBox *>("repoShowArchived");
        QVERIFY(checkbox && checkbox->isVisible());
        const auto verifyTick = [checkbox] {
            QStyleOptionButton option;
            option.initFrom(checkbox);
            option.state |= QStyle::State_On;
            const auto rect = checkbox->style()->subElementRect(QStyle::SE_CheckBoxIndicator, &option, checkbox);
            const auto image = checkbox->grab().toImage();
            const auto ratio = image.devicePixelRatio();
            const auto area = rect.adjusted(3, 3, -3, -3);
            const auto group = checkbox->isEnabled() ? QPalette::Active : QPalette::Disabled;
            const auto ink = checkbox->palette().color(group, QPalette::HighlightedText);
            const auto fill = checkbox->palette().color(group, QPalette::Highlight);
            int tick = 0, background = 0;
            for (int y = qRound(area.top() * ratio); y < qRound(area.bottom() * ratio); ++y) {
                for (int x = qRound(area.left() * ratio); x < qRound(area.right() * ratio); ++x) {
                    const auto color = image.pixelColor(x, y);
                    const auto distance = [color](const QColor &other) {
                        return qAbs(color.red() - other.red()) + qAbs(color.green() - other.green()) +
                               qAbs(color.blue() - other.blue());
                    };
                    tick += distance(ink) < 100 && distance(ink) < distance(fill);
                    background += distance(fill) < 45;
                }
            }
            return tick >= 2 && background >= 3;
        };
        const QPoint labelPoint(qMin(checkbox->width() - 2, 60), checkbox->height() / 2);
        QTest::mouseClick(checkbox, Qt::LeftButton, Qt::NoModifier, labelPoint);
        QTRY_VERIFY(checkbox->isChecked());
        QVERIFY(verifyTick());
        checkbox->setEnabled(false);
        QVERIFY(verifyTick());
        checkbox->setEnabled(true);
        QVERIFY(captureWidget(window, "repositories-checked-light"));
        themes->setCurrentIndex(themes->findData("dark"));
        checkbox->setFocus();
        QVERIFY(verifyTick());
        QVERIFY(captureWidget(window, "repositories-checked-dark"));
        QTest::keyClick(checkbox, Qt::Key_Space);
        QVERIFY(!checkbox->isChecked());

        settingsSection(window, "about");
        QVERIFY(window.findChild<QLabel *>("aboutDetails")->text().contains("HANDOFF_ONLY"));
        QVERIFY(window.findChild<QLabel *>("aboutDetails")->text().contains("ISOLATED_EXECUTION · BLOCKED"));
        QVERIFY(captureWidget(window, "settings-about"));
        auto *back = window.findChild<QPushButton *>("settingsBack");
        QTest::mouseClick(back, Qt::LeftButton);
        const auto before = invoke(client, "test.methods").value("items").toArray();
        verifyDashboardPages(window);
        const auto after = invoke(client, "test.methods").value("items").toArray();
        QVERIFY(!after.contains(QJsonValue("automation.templates")));
        QVERIFY(!after.contains(QJsonValue("release.check")));
        QVERIFY(after.size() >= before.size());
        auto *composer = window.findChild<QPlainTextEdit *>("conversationComposer");
        QVERIFY(composer);
        composer->setPlainText(tr("Draft kept while reviewing preferences"));
        settingsSection(window, "appearance");
        themes->setCurrentIndex(themes->findData("light"));
        themes->setCurrentIndex(themes->findData("dark"));
        QTest::keyClick(back, Qt::Key_Space);
        QTRY_VERIFY(composer->isVisible());
        QCOMPARE(composer->toPlainText(), tr("Draft kept while reviewing preferences"));
        composer->setFocus();
        QVERIFY(captureWidget(window, "chat-draft"));
        client.request("test.crash", {}, {});
        QTRY_VERIFY(!client.isRunning());
        auto *toast = window.findChild<QWidget *>("toast");
        QVERIFY(toast && toast->isVisible());
        QVERIFY(toast->geometry().top() < 32);
        QVERIFY(toast->parentWidget()->width() - toast->geometry().right() <= 20);
        QVERIFY(toast->height() <= 96);
        QCOMPARE(composer->toPlainText(), tr("Draft kept while reviewing preferences"));
        QVERIFY(captureWidget(window, "toast-disconnected"));
        QTest::mouseClick(toast->findChild<QToolButton *>("dismissToast"), Qt::LeftButton);
        QTRY_VERIFY(!toast->isVisible());
        settingsSection(window, "notifications");
        auto *history = window.findChild<QListWidget *>("notificationHistory");
        QVERIFY(history && history->count() >= 1);
        QVERIFY(captureWidget(window, "notification-history"));
        QTest::mouseClick(window.findChild<QPushButton *>("clearNotifications"), Qt::LeftButton);
        QCOMPARE(history->count(), 0);
        settingsSection(window, "runtime");
        auto *reconnect = window.findChild<QPushButton *>("reconnectRuntime");
        QTRY_VERIFY(reconnect && reconnect->isEnabled());
        QTest::keyClick(reconnect, Qt::Key_Space);
        QTRY_VERIFY(window.findChild<QLabel *>("runtimeState")->text().contains("connected"));
        QTest::keyClick(window.findChild<QPushButton *>("settingsBack"), Qt::Key_Space);
        QTRY_VERIFY(composer->isVisible() && !composer->isReadOnly());
        QCOMPARE(composer->toPlainText(), tr("Draft kept while reviewing preferences"));
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }

    void conversationHistoryPagesWithoutSidebarButton()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "paged-conversations"});
        MainWindow window(client);
        window.resize(1000, 580);
        window.show();
        client.start();
        auto *history = window.findChild<QWidget *>("conversationHistory");
        auto *list = history->findChild<QListWidget *>("conversationList");
        QTRY_COMPARE(list->count(), 30);
        for (auto *button : history->findChildren<QPushButton *>())
            QVERIFY(!button->isVisible() || button->text() != "Load more");

        list->setFocus();
        QTest::keyClick(list, Qt::Key_End);
        QTRY_COMPARE(list->count(), 60);
        QCOMPARE(list->item(59)->text(), QString("Fixture chat 60"));

        bool menuHasPagination = false;
        QTimer::singleShot(20, &window, [&] {
            auto *menu = qobject_cast<QMenu *>(QApplication::activePopupWidget());
            if (!menu)
                return;
            for (auto *action : menu->actions()) {
                if (action->text() == "Load more") {
                    menuHasPagination = true;
                    menu->setActiveAction(action);
                    QTest::keyClick(menu, Qt::Key_Return);
                    return;
                }
            }
            menu->close();
        });
        emit list->customContextMenuRequested(list->visualItemRect(list->currentItem()).center());
        QVERIFY(menuHasPagination);
        QTRY_COMPARE(list->count(), 65);
        QTest::keyClick(list, Qt::Key_End);
        QTest::qWait(80);
        const auto methods = invoke(client, "test.methods").value("items").toArray();
        QCOMPARE(std::count(methods.cbegin(), methods.cend(), QJsonValue("conversation.list")), 3);
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }

    void activeRuntimeTeardownDropsOutstandingReply()
    {
        bool called = false;
        {
            RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
            QSignalSpy ready(&client, &RuntimeClient::ready);
            client.start();
            QTRY_COMPARE(ready.size(), 1);
            client.request("repo.list", {}, [&called](const QJsonObject &, const QString &) { called = true; });
        }
        QVERIFY(!called);
    }

    void emptyAutomationAndPreferenceFailure()
    {
        RuntimeClient empty(QCoreApplication::applicationFilePath(), {"--fixture", "empty"});
        MainWindow window(empty);
        window.resize(1366, 768);
        window.show();
        empty.start();
        auto *emptyState = window.findChild<QLabel *>("automationState");
        QVERIFY(emptyState);
        QTRY_VERIFY(emptyState->isVisible() && !emptyState->text().isEmpty());
        QTRY_VERIFY(window.findChild<QPushButton *>("addAutomation")->isEnabled());
        QCOMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 0);
        QVERIFY(captureWidget(window, "automations-empty"));
        empty.stop();
        QTRY_VERIFY(!empty.isRunning());
        window.hide();

        RuntimeClient reject(QCoreApplication::applicationFilePath(), {"--fixture", "save-error"});
        MainWindow failed(reject);
        failed.show();
        reject.start();
        QTRY_VERIFY(failed.findChild<QPushButton *>("addAutomation")->isEnabled());
        settingsSection(failed, "appearance");
        auto *choice = failed.findChild<QComboBox *>("themeChoice");
        choice->setCurrentIndex(choice->findData("light"));
        QTRY_VERIFY(failed.findChild<QPushButton *>("preferencesRetry")->isVisible());
        QVERIFY(failed.findChild<QLabel *>("preferencesStatus")->text().contains("Could not save"));
        QCOMPARE(choice->currentData().toString(), QString("light"));
        reject.stop();
        QTRY_VERIFY(!reject.isRunning());
    }

    void notificationQueueAndQuietMode()
    {
        QWidget host;
        host.resize(660, 420);
        auto *editor = new QLineEdit(&host);
        editor->setGeometry(20, 20, 180, 30);
        auto *history = new QListWidget(&host);
        history->hide();
        NotificationCenter notices(host);
        notices.attachHistory(*history);
        host.show();
        host.activateWindow();
        editor->setFocus();
        notices.post("First", "info");
        notices.post("First", "info");
        notices.post("Second", "success");
        notices.post("Third", "warning");
        notices.post("Fourth", "error");
        QCOMPARE(history->count(), 4);
        QCOMPARE(host.findChildren<QWidget *>("toast").size(), 3);
        QCOMPARE(host.focusWidget(), editor);
        auto *first = host.findChildren<QWidget *>("toast").first();
        QTest::mouseClick(first->findChild<QToolButton *>("dismissToast"), Qt::LeftButton);
        QTRY_COMPARE(host.findChildren<QWidget *>("toast").size(), 3);
        QCOMPARE(host.findChildren<QWidget *>("toast").last()->property("severity").toString(), QString("error"));
        notices.setLevel("important");
        notices.post("Quiet success", "success");
        QCOMPARE(history->count(), 5);
        QTest::mouseClick(host.findChildren<QWidget *>("toast").first()->findChild<QToolButton *>("dismissToast"),
                         Qt::LeftButton);
        QTRY_COMPARE(host.findChildren<QWidget *>("toast").size(), 2);
        notices.markHistoryRead();
        notices.clearRead();
        QCOMPARE(history->count(), 0);
    }

    void pagedCatalogUsesBindingId()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "paged"});
        MainWindow window(client);
        window.resize(1366, 768);
        window.show();
        client.start();
        settingsSection(window, "repositories");
        auto *table = window.findChild<QTableView *>("repositoriesTable");
        auto *load = window.findChild<QPushButton *>("repoLoadMore");
        auto *search = window.findChild<QLineEdit *>("repoSearch");
        QVERIFY(table && load && search);
        QTRY_COMPARE(table->model()->rowCount(), 40);
        QVERIFY(load->isVisible() && load->isEnabled());
        QTest::keyClick(load, Qt::Key_Space);
        QTRY_COMPARE(table->model()->rowCount(), 41);
        table->sortByColumn(0, Qt::DescendingOrder);
        search->setText("Last page marker");
        QTRY_COMPARE(table->model()->rowCount(), 1);
        table->selectRow(0);
        const auto selected = repository_cases::selectedBinding(table);
        QVERIFY(selected.value("binding_id").toString().endsWith("29"));
        QTest::keyClick(window.findChild<QPushButton *>("repoRefresh"), Qt::Key_Space);
        QTRY_COMPARE(repository_cases::selectedBinding(table).value("binding_id"), selected.value("binding_id"));
        auto *use = window.findChild<QPushButton *>("repoUseBinding");
        QTRY_VERIFY(use && use->isEnabled());
        QTest::keyClick(use, Qt::Key_Space);
        QTRY_COMPARE(invoke(client, "test.lastSelection").value("binding_id"), selected.value("binding_id"));
        QTRY_VERIFY(window.findChild<QListWidget *>("repositoryPickerList"));
        auto *picker = window.findChild<QListWidget *>("repositoryPickerList");
        QTRY_COMPARE(picker->count(), 41);
        QCOMPARE(picker->selectedItems().size(), 1);
        QVERIFY(picker->selectedItems().first()->text().contains("Last page marker"));
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }

    void realRuntimePersistsRepositoryAndIntake()
    {
        const auto runtime = qEnvironmentVariable("CGACP_TEST_RUNTIME");
        if (runtime.isEmpty())
            QSKIP("Set CGACP_RUNTIME_EXECUTABLE at CMake configure time to run the Rust runtime test.");
        const auto git = QStandardPaths::findExecutable("git");
        QVERIFY(!git.isEmpty());
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        const auto directory = fixture.filePath(QStringLiteral("Mạch nguồn fixture"));
        QVERIFY(QDir().mkpath(directory));
        const auto source = QDir(directory).filePath(QStringLiteral("tệp mẫu.txt"));
        QFile file(source);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write("fixture source\n"), 15);
        file.close();
        QVERIFY(runGit(git, directory, {"init", "--quiet"}));
        QVERIFY(runGit(git, directory, {"add", "--", QFileInfo(source).fileName()}));
        QVERIFY(runGit(git, directory, {"-c", "user.name=CGACP Fixture", "-c", "user.email=fixture@example.invalid",
                                       "-c", "commit.gpgsign=false", "-c", "core.hooksPath=.git/nonexistent",
                                       "commit", "--quiet", "-m", "Fixture"}));
        RuntimeClient client(runtime, {"--stdio", "--data-dir", fixture.filePath("database")});
        MainWindow window(client);
        window.resize(1366, 768);
        window.show();
        QSignalSpy connected(&client, &RuntimeClient::ready);
        client.start();
        QTRY_COMPARE(connected.size(), 1);
        const auto info = invoke(client, "runtime.info");
        QCOMPARE(info.value("product_generation").toString(), QStringLiteral(CGACP_PRODUCT_GENERATION));
        const auto preview = invoke(client, "repo.preview", {{"path", source}});
        QCOMPARE(preview.value("file_hint").toString(), QFileInfo(source).fileName());
        QVERIFY(preview.value("root").toString().contains(QStringLiteral("Mạch nguồn fixture")));

        settingsSection(window, "repositories");
        auto *picker = window.findChild<QAction *>("selectRepositoryFile");
        QVERIFY(picker);
        QTimer drivePicker;
        drivePicker.setInterval(20);
        bool confirmed = false;
        connect(&drivePicker, &QTimer::timeout, &window, [&] {
            if (auto *fileDialog = qobject_cast<QFileDialog *>(QApplication::activeModalWidget())) {
                fileDialog->setDirectory(directory);
                fileDialog->selectFile(QFileInfo(source).fileName());
                QMetaObject::invokeMethod(fileDialog, "accept");
            } else if (auto *dialog = qobject_cast<QDialog *>(QApplication::activeModalWidget());
                       dialog && dialog->windowTitle() == "Confirm repository") {
                if (auto *buttons = dialog->findChild<QDialogButtonBox *>()) {
                    auto *save = buttons->button(QDialogButtonBox::Save);
                    if (save && save->isEnabled()) {
                        QVERIFY(captureWidget(*dialog, "repository-confirmation-real-runtime"));
                        confirmed = true;
                        save->click();
                    }
                }
            }
        });
        QTimer::singleShot(4500, &drivePicker, [&] {
            if (!confirmed) {
                if (auto *dialog = qobject_cast<QDialog *>(QApplication::activeModalWidget()))
                    dialog->reject();
            }
        });
        drivePicker.start();
        picker->trigger();
        QTRY_VERIFY(confirmed);
        drivePicker.stop();
        auto *table = window.findChild<QTableView *>("repositoriesTable");
        QVERIFY(table);
        QTRY_COMPARE(table->model()->rowCount(), 1);
        const auto binding = invoke(client, "repo.list").value("items").toArray().first().toObject();
        const auto bindingId = binding.value("binding_id").toString();
        QVERIFY(!bindingId.isEmpty());
        repository_cases::exerciseTable(window, binding);
        verifyLegacyAutomation(window, bindingId, QFileInfo(source).fileName(),
            [&client](const QString &method, QJsonObject params) { return invoke(client, method, std::move(params)); });
        const auto conversation = invoke(client, "conversation.create", {{"title", "Temporary UI task"},
            {"bindings", QJsonArray{bindingId}}, {"profile", "handoff"}, {"key", "desktop-fixture-conversation"}});
        const auto conversationId = conversation.value("id").toString(conversation.value("conversation").toObject().value("id").toString());
        QVERIFY(!conversationId.isEmpty());
        const QJsonObject send{{"id", conversationId}, {"content", "Check the fixture interface"},
                               {"key", "desktop-fixture-message"}};
        const auto accepted = invoke(client, "conversation.send", send);
        const auto retry = invoke(client, "conversation.send", send);
        QVERIFY(!accepted.value("message").toObject().value("id").toString().isEmpty());
        QCOMPARE(accepted.value("message").toObject().value("id"), retry.value("message").toObject().value("id"));
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
        client.start();
        QTRY_COMPARE(connected.size(), 2);
        const auto bindings = invoke(client, "repo.list").value("items").toArray();
        QCOMPARE(bindings.size(), 1);
        QCOMPARE(bindings[0].toObject().value("binding_id").toString(), bindingId);
        const auto saved = invoke(client, "conversation.get", {{"id", conversationId}});
        QCOMPARE(saved.value("messages").toArray().size(), 1);
        QCOMPARE(saved.value("messages").toArray()[0].toObject().value("content").toString(),
                 QStringLiteral("Check the fixture interface"));
        const auto relocated = fixture.filePath(QStringLiteral("Đã chuyển worktree"));
        QVERIFY(QDir().rename(directory, relocated));
        QCOMPARE(invoke(client, "repo.health", {{"binding_id", bindingId}}).value("status").toString(),
                 QStringLiteral("MISSING"));
        const auto relocation = invoke(client, "repo.preview", {{"path", relocated}});
        const auto relink = invoke(client, "repo.relink", {{"binding_id", bindingId},
            {"revision", bindings[0].toObject().value("revision")}, {"path", relocated}, {"token", relocation.value("token")}});
        QCOMPARE(relink.value("binding_id").toString(), bindingId);
        QCOMPARE(relink.value("revision").toInteger(), bindings[0].toObject().value("revision").toInteger() + 1);
        settingsSection(window, "repositories");
        QTest::mouseClick(window.findChild<QPushButton *>("repoRefresh"), Qt::LeftButton);
        QTRY_COMPARE(table->model()->rowCount(), 1);
        QTRY_VERIFY(table->model()->index(0, 1).data().toString().contains(QStringLiteral("Đã chuyển worktree")));
        for (int count = 0; count < 15; ++count) {
            const auto toasts = window.findChildren<QWidget *>("toast");
            const auto visible = std::find_if(toasts.cbegin(), toasts.cend(), [](auto *toast) { return toast->isVisible(); });
            if (visible == toasts.cend())
                break;
            QTest::keyClick((*visible)->findChild<QToolButton *>("dismissToast"), Qt::Key_Space);
        }
        table->horizontalHeader()->resizeSection(1, 420);
        QVERIFY(captureWidget(window, "repositories-real-runtime"));
        repository_cases::exerciseArchive(window, bindingId, QDir(relocated).filePath(QFileInfo(source).fileName()));

        auto *header = table->horizontalHeader();
        header->resizeSection(1, 1200);
        header->moveSection(header->visualIndex(1), 0);
        table->sortByColumn(2, Qt::DescendingOrder);
        QTRY_VERIFY(table->horizontalScrollBar()->maximum() >= 65);
        table->horizontalScrollBar()->setSliderPosition(65);
        auto *expandedSplit = window.findChild<QSplitter *>("shellSplitter");
        auto *handle = expandedSplit->handle(1);
        const auto center = handle->rect().center();
        const int beforeDrag = expandedSplit->sizes().first();
        QTest::mousePress(handle, Qt::LeftButton, Qt::NoModifier, center);
        QTest::mouseMove(handle, center + QPoint(31, 0), 30);
        QTest::mouseRelease(handle, Qt::LeftButton, Qt::NoModifier, center);
        const int savedWidth = expandedSplit->sizes().first();
        QVERIFY(savedWidth > beforeDrag);
        settingsSection(window, "appearance");
        auto *themeChoice = window.findChild<QComboBox *>("themeChoice");
        themeChoice->setCurrentIndex(themeChoice->findData("system"));
        QTest::mouseClick(window.findChild<QToolButton *>("toggleSidebar"), Qt::LeftButton);
        QTRY_COMPARE(invoke(client, "preferences.get").value("settings").toObject().value("theme").toString(), QString("system"));
        QTRY_VERIFY(invoke(client, "preferences.get").value("settings").toObject().value("sidebar_collapsed").toBool());
        QCOMPARE(invoke(client, "preferences.get").value("settings").toObject().value("repository_layout")
                     .toObject().value("horizontal_scroll").toInt(), 65);
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
        window.hide();
        RuntimeClient reopenedClient(runtime, {"--stdio", "--data-dir", fixture.filePath("database")});
        MainWindow reopened(reopenedClient);
        reopened.resize(1366, 768);
        reopened.show();
        reopenedClient.start();
        QTRY_COMPARE(reopened.findChild<QComboBox *>("themeChoice")->currentData().toString(), QString("system"));
        const auto scheme = QGuiApplication::styleHints()->colorScheme();
        if (scheme != Qt::ColorScheme::Unknown)
            QCOMPARE(QApplication::palette().color(QPalette::Window).lightness() > 128, scheme == Qt::ColorScheme::Light);
        auto *split = reopened.findChild<QSplitter *>("shellSplitter");
        QCOMPARE(split->sizes().first(), 60);
        settingsSection(reopened, "repositories");
        QVERIFY(reopened.findChild<QCheckBox *>("repoShowArchived")->isChecked());
        auto *reopenedTable = reopened.findChild<QTableView *>("repositoriesTable");
        QCOMPARE(reopenedTable->horizontalHeader()->sectionSize(1), 1200);
        QCOMPARE(reopenedTable->horizontalHeader()->logicalIndex(0), 1);
        QCOMPARE(reopenedTable->horizontalHeader()->sortIndicatorSection(), 2);
        QCOMPARE(reopenedTable->horizontalHeader()->sortIndicatorOrder(), Qt::DescendingOrder);
        QTRY_COMPARE(repository_cases::selectedBinding(reopenedTable).value("binding_id").toString(), bindingId);
        QTRY_COMPARE(reopenedTable->horizontalScrollBar()->value(), 65);
        QTest::keyClick(reopened.findChild<QToolButton *>("toggleSidebar"), Qt::Key_Space);
        QCOMPARE(split->sizes().first(), savedWidth);
        settingsSection(reopened, "appearance");
        QTest::keyClick(reopened.findChild<QPushButton *>("resetLayout"), Qt::Key_Space);
        QCOMPARE(reopenedTable->horizontalHeader()->sectionSize(1), 290);
        const auto afterReset = invoke(reopenedClient, "repo.list", {{"include_archived", true}}).value("items").toArray();
        QCOMPARE(afterReset.size(), 1);
        QCOMPARE(afterReset[0].toObject().value("binding_id").toString(), bindingId);
        reopenedClient.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!reopenedClient.isRunning(), 6000);
    }

    void vietnameseCatalogLoads()
    {
        QTranslator translator;
        const auto file = qEnvironmentVariable("CGACP_TEST_QM");
        QVERIFY(!file.isEmpty());
        QVERIFY(translator.load(file));
        QCOMPARE(translator.translate("MainWindow", "Automations"), QStringLiteral("Tự động hóa"));
        QVERIFY(!translator.translate("MainWindow", "Appearance").isEmpty());
        QCOMPARE(translator.translate("MainWindow", "Cancel pending task"), QStringLiteral("Hủy nhiệm vụ đang chờ"));
    }

    void vietnameseNativeLayout()
    {
        QTranslator translator;
        QVERIFY(translator.load(qEnvironmentVariable("CGACP_TEST_QM")));
        struct RestoreLanguage {
            QTranslator &translator;
            ~RestoreLanguage() { QCoreApplication::removeTranslator(&translator); }
        } restore{translator};
        QCoreApplication::installTranslator(&translator);
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        MainWindow window(client);
        const auto ratio = window.devicePixelRatioF();
        window.resize(qRound(1366 / ratio), qRound(768 / ratio));
        window.show();
        client.start();
        QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 3);
        auto *nav = window.findChild<QPushButton *>("navAutomations");
        QCOMPARE(nav->text(), QStringLiteral("Tự động hóa"));
        QVERIFY(nav->fontMetrics().horizontalAdvance(nav->text()) + 44 <= nav->width());
        QVERIFY(captureWidget(window, "vi-automations"));
        settingsSection(window, "appearance");
        auto *scaling = window.findChild<QSpinBox *>("textScale");
        QVERIFY(scaling);
        scaling->setValue(125);
        auto *brand = window.findChild<QLabel *>("brand");
        auto *appearance = window.findChild<QWidget *>("appearancePage");
        QVERIFY(brand && appearance);
        QTRY_VERIFY(brand->width() >= brand->fontMetrics().horizontalAdvance(brand->text()));
        for (auto *label : appearance->findChildren<QLabel *>()) {
            if (label->buddy())
                QTRY_VERIFY(label->width() >= label->fontMetrics().horizontalAdvance(label->text().remove('&')));
        }
        auto *reset = window.findChild<QPushButton *>("resetLayout");
        QTRY_VERIFY(reset->width() >= reset->fontMetrics().horizontalAdvance(reset->text()) + 16);
        QVERIFY(captureWidget(window, "vi-appearance-text125"));
        auto *header = window.findChild<QComboBox *>("settingsSection");
        QVERIFY(header->fontMetrics().horizontalAdvance(header->currentText()) + 18 <= header->width());
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }
};

int main(int argc, char **argv)
{
    if (argc >= 2 && QByteArray(argv[1]) == "--fixture")
        return runFixture(argc == 3 ? QString::fromLatin1(argv[2]) : QString{});
    QApplication::setAttribute(Qt::AA_DontUseNativeDialogs);
    QApplication app(argc, argv);
    const auto fontPath = qEnvironmentVariable("CGACP_TEST_FONT");
    if (!fontPath.isEmpty()) {
        const auto id = QFontDatabase::addApplicationFont(fontPath);
        const auto families = QFontDatabase::applicationFontFamilies(id);
        if (families.isEmpty())
            return 19;
        app.setFont(QFont(families.first()));
    }
    applyTheme(app);
    DesktopTest test;
    return QTest::qExec(&test, argc, argv);
}

#include "desktop_test.moc"
