#include "build_info.h"
#include "main_window.h"
#include "runtime_client.h"
#include "notification_center.h"
#include "pages_test.h"
#include "repository_cases.h"
#include "navigation_test.h"
#include "notification_visual_test.h"
#include "settings_polish_test.h"
#include "table_visual_test.h"
#include "interaction_cases.h"
#include "shared_polish_test.h"

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
#include <QRadioButton>
#include <QScreen>
#include <QScrollArea>
#include <QScrollBar>
#include <QSpinBox>
#include <QStyleHints>
#include <QSignalSpy>
#include <QSplitter>
#include <QStackedWidget>
#include <QStandardPaths>
#include <QStyleOption>
#include <QTest>
#include <QTemporaryDir>
#include <QTableView>
#include <QTextCursor>
#include <QTextDocument>
#include <QTimer>
#include <QToolButton>
#include <QToolTip>
#include <QTranslator>
#include <QWindow>
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
    if (method == "preferences.get") {
        QJsonObject settings{{"theme", "dark"}, {"density", "comfortable"},
                {"text_scale", 100}, {"sidebar_width", 260}, {"sidebar_collapsed", false},
                {"repository_layout", QJsonValue(QJsonValue::Null)}, {"notification_level", "all"}};
        if (mode == "startup-language")
            settings.insert("appearance", QJsonObject{{"language", "zh_CN"}});
        return {{"revision", 0}, {"settings", settings}};
    }
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
    if (mode == "transcript-paged" && method == "conversation.get") {
        const bool earlier = params.value("before").toString() == "3";
        const auto after = params.value("after").toString();
        const int start = earlier ? 1 : after.isEmpty() ? 3 : after == "4" ? 5 : 6;
        const int end = earlier ? 2 : after.isEmpty() ? 4 : 5;
        QJsonArray messages;
        for (int ordinal = start; ordinal <= end; ++ordinal)
            messages.append(QJsonObject{{"id", QString::number(ordinal)}, {"ordinal", ordinal},
                                        {"role", "USER"}, {"content", QString("Message %1").arg(ordinal)}});
        return {{"conversation", QJsonObject{{"id", params.value("id")}, {"title", "Fixture chat"},
                                               {"state", "ACTIVE"}, {"revision", 1}}},
                {"messages", messages}, {"next_before", after.isEmpty() && !earlier ? QJsonValue("3") : QJsonValue()},
                {"has_more", false}};
    }
    if (mode == "transcript-paged" && method == "conversation.events")
        return {{"next_after", "1"}, {"items", QJsonArray{QJsonObject{{"type", "MESSAGE_SAVED"}}}}};
    if (mode == "automation-cascade" && method == "conversation.delete.preview")
        return {{"title", "Fixture chat"}, {"revision", 1}, {"token", "fixture-preview"},
                {"automation_count", 1}, {"automations", QJsonArray{QJsonObject{
                    {"id", "100"}, {"name", "Shared title"}, {"revision", 7}, {"enabled", false}}}}};
    if (method == "conversation.get")
        return {{"conversation", QJsonObject{{"id", params.value("id")}, {"title", params.value("id") == "chat-1" ?
                             "Fixture chat" : "Other chat"}, {"state", "ACTIVE"}, {"revision", 1}}},
                {"messages", QJsonArray{QJsonObject{{"role", "USER"}, {"content", "Saved fixture message"}}}}};
    if (method == "automation.list" && (mode == "automation-delete" || mode == "automation-cascade")) {
        QJsonArray items;
        if (!methods.contains("automation.delete") && !methods.contains("conversation.delete"))
            items.append(QJsonObject{{"id", "100"}, {"name", "Shared title"}, {"prompt", "Target automation"},
                                     {"enabled", false}, {"revision", 7}});
        items.append(QJsonObject{{"id", "101"}, {"name", "Shared title"}, {"prompt", "Independent automation"},
                                 {"enabled", false}, {"revision", 3}});
        return {{"items", items}};
    }
    if (method == "automation.delete" || method == "conversation.delete")
        return {{"id", params.value("id")}, {"status", "DELETED"}};
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
    QList<QByteArray> heldReplies;
    bool firstPreferenceWrite = true;
    bool holdAutomationList = false;
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
        if (method == "test.slow")
            QTest::qSleep(2300);
        const auto params = request.value("params").toObject();
        if (method == "repo.select")
            lastRepoSelection = params;
        if (method == "test.holdAutomationList")
            holdAutomationList = true;
        methods.append(method);
        QJsonObject responseObject{{"v", 1}, {"id", request.value("id")}};
        if (mode == "save-error" && method == "preferences.set" && firstPreferenceWrite) {
            firstPreferenceWrite = false;
            responseObject.insert("ok", false);
            responseObject.insert("error", QJsonObject{{"code", "STORAGE_UNAVAILABLE"}, {"message", "Fixture write denied."}});
        } else if (mode == "automation-delete" && method == "automation.delete"
                   && (params.value("id") != "100" || params.value("revision") != 7)) {
            responseObject.insert("ok", false);
            responseObject.insert("error", QJsonObject{{"code", "INVALID_REQUEST"}, {"message", "Wrong automation scope"}});
        } else {
            responseObject.insert("ok", true);
            responseObject.insert("result", method == "test.lastSelection" ? lastRepoSelection :
                                  fixtureResult(method, params, methods, mode));
        }
        const auto response = QJsonDocument(responseObject).toJson(QJsonDocument::Compact) + '\n';
        if (method == "automation.list" && holdAutomationList) {
            holdAutomationList = false;
            heldReplies.append(response);
            continue;
        }
        if (mode == "eof-burst") {
            heldReplies.append(response);
            continue;
        }
        if (mode == "navigation-delayed" &&
            ((method == "conversation.get" && params.value("id") == "chat-1") ||
             (method == "conversation.list" && !params.value("query").toString().isEmpty()))) {
            heldReplies.append(response);
            continue;
        }
        if (method == "test.releaseNavigation") {
            for (const auto &reply : heldReplies)
                std::fwrite(reply.constData(), 1, size_t(reply.size()), stdout);
            heldReplies.clear();
        }
        const auto half = response.size() / 2;
        std::fwrite(response.data(), 1, size_t(half), stdout);
        std::fflush(stdout);
        QTest::qSleep(7);
        std::fwrite(response.data() + half, 1, size_t(response.size() - half), stdout);
        std::fflush(stdout);
    }
    if (mode == "eof-burst") {
        for (const auto &reply : heldReplies)
            std::fwrite(reply.constData(), 1, size_t(reply.size()), stdout);
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
    auto *choice = window.findChild<QListWidget *>("settingsNavigation");
    QVERIFY(choice && choice->isVisible());
    for (int row = 0; row < choice->count(); ++row) {
        if (choice->item(row)->data(Qt::UserRole).toString() == section) {
            choice->setCurrentRow(row);
            return;
        }
    }
    QFAIL("Settings section is missing");
}

void chooseTheme(MainWindow &window, const QString &name)
{
    auto *tile = window.findChild<QRadioButton *>(QStringLiteral("theme") + name);
    QVERIFY(tile);
    tile->setChecked(true);
}

bool captureWidget(QWidget &widget, const QString &name)
{
    const auto directory = qEnvironmentVariable("CGACP_UI_CAPTURE");
    if (directory.isEmpty())
        return true;
    QTest::qWait(60);
    return widget.grab().save(directory + "/" + name + ".png");
}

void withDashboardFixture(const std::function<void(MainWindow &)> &test,
                          const std::function<void(RuntimeClient &)> &after = {})
{
    RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
    MainWindow window(client);
    window.resize(1366, 768);
    interaction_cases::pinToPrimary(window);
    window.show();
    client.start();
    QTRY_VERIFY(window.findChild<QPushButton *>("addAutomation")->isEnabled());
    test(window);
    if (after)
        after(client);
    client.stop();
    QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
}
}

class DesktopTest final : public QObject {
    Q_OBJECT
private slots:
    void scrollbarsRevealOnThumbHoverAndWheel() { shared_polish::scrollbarsRevealOnThumbHoverAndWheel(); }
    void appearancePixelsAndPaletteApplyToOpenWidgets() { shared_polish::appearancePixelsAndPaletteApplyToOpenWidgets(); }
    void wheelScrollsWithoutChangingEditors() { shared_polish::wheelScrollsWithoutChangingEditors(); }

    void automationDeletionUsesIdAndRevision()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "automation-delete"});
        MainWindow window(client);
        window.show();
        client.start();
        const auto cardBody = [&window](const QString &prompt) -> QLabel * {
            for (auto *body : window.findChildren<QLabel *>("cardBody"))
                if (body->text() == prompt)
                    return body;
            return nullptr;
        };
        QTRY_VERIFY(cardBody("Target automation") && cardBody("Independent automation"));
        auto *remove = cardBody("Target automation")->parentWidget()->findChild<QAction *>("deleteAutomationDefinition");
        QVERIFY(remove && remove->isEnabled());
        remove->trigger();
        QTRY_VERIFY(window.findChild<QMessageBox *>("deleteAutomationConfirmation"));
        auto *confirmation = window.findChild<QMessageBox *>("deleteAutomationConfirmation");
        QVERIFY(confirmation->text().contains("100"));
        QVERIFY(captureWidget(*confirmation, "delete-automation-confirmation"));
        confirmation->button(QMessageBox::Yes)->click();
        QTRY_VERIFY(!cardBody("Target automation"));
        QVERIFY(cardBody("Independent automation"));
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }

    void conversationDeletionRefreshesAutomationWithoutReplayingStalePage()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "automation-cascade"});
        MainWindow window(client);
        window.show();
        client.start();
        QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 2);
        auto *history = window.findChild<QListWidget *>("conversationList");
        auto *editor = window.findChild<QPlainTextEdit *>("conversationComposer");
        auto *refresh = window.findChild<QPushButton *>("refreshAutomations");
        QVERIFY(history && editor && refresh);
        QTRY_VERIFY(history->count() > 0);
        QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                          history->visualItemRect(history->item(0)).center());
        QTRY_VERIFY(editor->isVisible() && !editor->isReadOnly());
        auto *remove = window.findChild<QAction *>("deleteConversation");
        QVERIFY(remove);
        remove->trigger();
        QTRY_VERIFY(window.findChild<QMessageBox *>("deleteDataConfirmation"));
        auto *confirmation = window.findChild<QMessageBox *>("deleteDataConfirmation");
        QVERIFY(confirmation->text().contains("Shared title"));
        QVERIFY(confirmation->text().contains("100"));
        QVERIFY(captureWidget(*confirmation, "conversation-automation-delete-confirmation"));
        invoke(client, "test.holdAutomationList");
        refresh->click();
        confirmation->button(QMessageBox::Yes)->click();
        QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 1);
        invoke(client, "test.releaseNavigation");
        QVERIFY(!invoke(client, "test.methods").value("items").toArray().contains("automation.delete"));
        QCOMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 1);
        QCOMPARE(window.findChildren<QLabel *>("automationCardTitle").first()->parentWidget()
                     ->findChild<QLabel *>("cardBody")->text(), QString("Independent automation"));
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }

    void closingRuntimeAppliesAllBufferedReplies()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "eof-burst"});
        QSignalSpy ready(&client, &RuntimeClient::ready);
        QSignalSpy stopped(&client, &RuntimeClient::stopped);
        client.start();
        QTRY_COMPARE(ready.size(), 1);
        int accepted = 0;
        QStringList failures;
        for (int index = 0; index < 32; ++index)
            client.request("runtime.info", {}, [&](const QJsonObject &, const QString &failure) {
                if (failure.isEmpty())
                    ++accepted;
                else
                    failures.append(failure);
            });
        client.stop();
        QTRY_COMPARE(stopped.size(), 1);
        QCOMPARE(accepted, 32);
        QVERIFY(failures.isEmpty());
    }

    void transcriptPagesAppendEventsWithoutReset()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "transcript-paged"});
        MainWindow window(client);
        window.show();
        client.start();
        auto *history = window.findChild<QListWidget *>("conversationList");
        auto *transcript = window.findChild<QPlainTextEdit *>("conversationTranscript");
        auto *older = window.findChild<QPushButton *>("loadOlderMessages");
        auto *poll = window.findChild<QTimer *>("conversationEventPoll");
        QVERIFY(history && transcript && older && poll);
        QTRY_VERIFY(history->count() > 0);
        QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                          history->visualItemRect(history->item(0)).center());
        QTRY_VERIFY(transcript->toPlainText().contains("Message 4"));
        QVERIFY(!transcript->toPlainText().contains("Message 1"));
        QTRY_VERIFY(older->isVisible() && older->isEnabled());
        older->click();
        QTRY_VERIFY(transcript->toPlainText().startsWith("You\nMessage 1"));
        const auto before = transcript->toPlainText();
        QSignalSpy edits(transcript->document(), &QTextDocument::contentsChange);
        QVERIFY(QMetaObject::invokeMethod(poll, "timeout"));
        QTRY_VERIFY(transcript->toPlainText().endsWith("Message 5"));
        QVERIFY(transcript->toPlainText().startsWith(before));
        QVERIFY(!edits.isEmpty());
        for (const auto &change : edits) {
            QVERIFY(change.at(0).toInt() >= before.size());
            QCOMPARE(change.at(1).toInt(), 0);
        }
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }

    void desktopRouteLoad()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        MainWindow window(client);
        window.resize(1200, 700);
        interaction_cases::pinToPrimary(window);
        window.show();
        client.start();
        auto *history = window.findChild<QListWidget *>("conversationList");
        auto *composer = window.findChild<QPlainTextEdit *>("conversationComposer");
        auto *automations = window.findChild<QPushButton *>("navAutomations");
        auto *settings = window.findChild<QToolButton *>("navSettings");
        auto *back = window.findChild<QPushButton *>("settingsBack");
        QVERIFY(history && composer && automations && settings && back);
        QTRY_VERIFY(history->count() > 0);
        const auto open = [&] {
            QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                              history->visualItemRect(history->item(0)).center());
        };
        open();
        QTRY_VERIFY(composer->isVisible() && !composer->isReadOnly());
        const auto cycle = [&] {
            settings->click();
            QVERIFY(back->isVisible());
            back->click();
            QVERIFY(composer->isVisible());
            automations->click();
            QVERIFY(automations->isChecked());
            open();
            QVERIFY(composer->isVisible());
        };
        for (int index = 0; index < 10; ++index)
            cycle();
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        const auto objectsBefore = window.findChildren<QObject *>().size();
        const auto widgetsBefore = window.findChildren<QWidget *>().size();
        const auto timersBefore = client.findChildren<QTimer *>().size() + window.findChildren<QTimer *>().size();
        bool finished = false;
        QString failure;
        QElapsedTimer roundTrip;
        roundTrip.start();
        qint64 responseMs = -1;
        client.request("test.slow", {}, [&](const QJsonObject &, const QString &error) {
            responseMs = roundTrip.elapsed();
            failure = error;
            finished = true;
        });
        composer->setFocus();
        QTest::keyClicks(composer, "Typing while backend is delayed");
        QVERIFY(!finished);
        QList<double> samples;
        for (int index = 0; index < 100; ++index) {
            QElapsedTimer elapsed;
            elapsed.start();
            cycle();
            samples.append(elapsed.nsecsElapsed() / 1.0e6);
            QTest::qWait(1);
        }
        QCOMPARE(composer->toPlainText(), QStringLiteral("Typing while backend is delayed"));
        QTRY_VERIFY_WITH_TIMEOUT(finished, 5000);
        QVERIFY(failure.isEmpty());
        QVERIFY(responseMs >= 2000);
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        std::sort(samples.begin(), samples.end());
        qInfo() << "route_load cycles" << samples.size() << "logical_window" << window.size()
                << "DPR" << window.devicePixelRatioF() << "ms p50 p95 p99 max"
                << samples[49] << samples[94] << samples[98] << samples.last()
                << "slow_ipc_ms" << responseMs << "objects"
                << objectsBefore << window.findChildren<QObject *>().size()
                << "widgets" << widgetsBefore << window.findChildren<QWidget *>().size()
                << "timers" << timersBefore
                << client.findChildren<QTimer *>().size() + window.findChildren<QTimer *>().size();
        QVERIFY(samples[94] <= 100);
        QVERIFY(samples.last() <= 250);
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }

    void iconStyles() { interaction_cases::iconStyles(); }
    void toastAlignmentAndSeverities() { notification_visual::verifyAlignmentAndSeverities(); }

    void conversationMovesExistingToastsToUpperRight()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        MainWindow window(client);
        window.resize(1200, 700);
        interaction_cases::pinToPrimary(window);
        window.show();
        client.start();
        notification_visual::verifyConversationPlacement(window);
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }

    void localReportsStatusWithoutNavigation()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        MainWindow window(client);
        interaction_cases::pinToPrimary(window);
        window.show();
        client.start();
        auto *local = window.findChild<QToolButton *>("localStatus");
        auto *pages = window.findChild<QStackedWidget *>("pages");
        QVERIFY(local && pages);
        QTRY_COMPARE(local->toolTip(), QStringLiteral("Local runtime connected"));
        QVERIFY2(!local->menu(), "Local must report status without opening a menu");
        const auto *page = pages->currentWidget();
        QTest::mouseClick(local, Qt::LeftButton);
        QTRY_COMPARE(QToolTip::text(), local->toolTip());
        QCOMPARE(pages->currentWidget(), page);
        QVERIFY(!QApplication::activePopupWidget());
        QVERIFY(captureWidget(window, "local-connected-status"));
        for (auto *popup : QApplication::topLevelWidgets()) {
            if (popup->windowType() == Qt::ToolTip && popup->isVisible())
                QVERIFY(captureWidget(*popup, "local-connected-tooltip"));
        }

        QToolTip::hideText();
        auto *toggle = window.findChild<QToolButton *>("toggleSidebar");
        QVERIFY(toggle);
        QTest::mouseClick(toggle, Qt::LeftButton);
        QVERIFY(local->text().isEmpty());
        QCOMPARE(local->accessibleName(), QStringLiteral("Local — Local runtime connected"));
        client.request("test.crash", {}, {});
        QTRY_VERIFY(!client.isRunning());
        QTRY_VERIFY(local->toolTip() != QStringLiteral("Local runtime connected"));
        const auto failure = local->toolTip();
        QVERIFY(!failure.isEmpty());
        QTest::keyClick(local, Qt::Key_Space);
        QTRY_COMPARE(QToolTip::text(), failure);
        QCOMPARE(pages->currentWidget(), page);
        QVERIFY(captureWidget(window, "local-disconnected-status"));
        QToolTip::hideText();
    }

    void mouseAndKeyboardFocus() { interaction_cases::mouseAndKeyboardFocus(); }
    void compoundArrows() { interaction_cases::compoundArrows(); }
    void settingsLayoutAndKeyboard() { withDashboardFixture(verifySettingsPolish); }
    void languageUpdatesOpenDashboard()
    {
        withDashboardFixture([](MainWindow &window) {
            settingsSection(window, "appearance");
            auto *choice = window.findChild<QComboBox *>("languageChoice");
            auto *search = window.findChild<QLineEdit *>("settingsSearch");
            auto *navigation = window.findChild<QListWidget *>("settingsNavigation");
            auto *draft = window.findChild<QPlainTextEdit *>("conversationComposer");
            QVERIFY(choice && search && navigation && draft);
            const QSignalBlocker draftSignals(draft);
            draft->setPlainText("A draft that must survive changing languages");
            choice->setCurrentIndex(choice->findData("vi"));
            QTRY_COMPARE(search->placeholderText(), QStringLiteral("Tìm trong cài đặt"));
            QTRY_COMPARE(navigation->currentItem()->text(), QStringLiteral("Giao diện"));
            choice->setCurrentIndex(choice->findData("zh_CN"));
            QTRY_COMPARE(search->placeholderText(), QStringLiteral("搜索设置"));
            settingsSection(window, "storage");
            QCOMPARE(window.findChild<QPushButton *>("refreshChatStorage")->text(), QStringLiteral("刷新"));
            QVERIFY(captureWidget(window, "storage-live-chinese"));
            settingsSection(window, "appearance");
            choice->setCurrentIndex(choice->findData("en"));
            QTRY_COMPARE(search->placeholderText(), QStringLiteral("Search settings"));
            QCOMPARE(draft->toPlainText(), QStringLiteral("A draft that must survive changing languages"));
        });
    }

    void storageMaintenanceGroupsActions()
    {
        withDashboardFixture([](MainWindow &window) {
            settingsSection(window, "storage");
            auto *maintenance = window.findChild<QPushButton *>("storageMaintenanceToggle");
            auto *backup = window.findChild<QPushButton *>("createStorageBackup");
            auto *cancel = window.findChild<QPushButton *>("cancelStorageOperation");
            QVERIFY(maintenance && backup && cancel);
            QVERIFY(!backup->isVisible());
            maintenance->click();
            QVERIFY(backup->isVisible());
            QVERIFY(backup->property("primary").toBool());
            QVERIFY(window.findChild<QWidget *>("storageDatabaseActions"));
            QVERIFY(window.findChild<QWidget *>("storageBackupActions"));
            QVERIFY(!cancel->isVisible());
            QVERIFY(captureWidget(window, "storage-maintenance"));
            window.resize(760, 680);
            QTRY_VERIFY(backup->mapTo(&window, QPoint(backup->width(), 0)).x() <= window.width());
        });
    }

    void modeFocusIsSolidOnCustomBackground()
    {
        withDashboardFixture([](MainWindow &window) {
            settingsSection(window, "appearance");
            auto *background = window.findChild<QLineEdit *>("appearanceBackground");
            auto *dark = window.findChild<QRadioButton *>("themeDark");
            QVERIFY(background && dark);
            background->setText("#203040");
            QTest::keyClick(background, Qt::Key_Return);
            QTRY_COMPARE(window.palette().color(QPalette::Window), QColor("#203040"));
            dark->setProperty("keyboardFocusVisible", true);
            const auto sample = dark->grab();
            const auto pixels = sample.toImage();
            const auto ratio = sample.devicePixelRatio();
            const auto y = qRound(1.5 * ratio);
            for (int x = 18; x < dark->width() - 18; ++x)
                QCOMPARE(pixels.pixelColor(qRound(x * ratio), y), dark->palette().color(QPalette::Link));
            QVERIFY(captureWidget(window, "appearance-solid-focus"));
        });
    }
    void exclusiveNavigation() { withDashboardFixture(verifyExclusiveNavigation); }
    void newChatCancelKeepsRoute()
    {
        withDashboardFixture(verifyNewChatCancelKeepsRoute, [](RuntimeClient &client) {
            QVERIFY(!invoke(client, "test.methods").value("items").toArray().contains(QJsonValue("conversation.create")));
        });
    }
    void collapsedSidebarGrouping() { withDashboardFixture(verifyCollapsedSidebarGrouping); }

    void lateConversationReplies()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "navigation-delayed"});
        MainWindow window(client);
        window.resize(1366, 768);
        interaction_cases::pinToPrimary(window);
        window.show();
        client.start();
        verifyLateConversationReplies(window, [&client](const QString &method) { return invoke(client, method); });
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }

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
        interaction_cases::pinToPrimary(window);
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
        QCOMPARE(window.findChild<QToolButton *>("localStatus")->text(), QString("Local"));
        QVERIFY(captureWidget(*footer, "sidebar-footer"));
        QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 3);
        const auto initialMethods = invoke(client, "test.methods").value("items").toArray();
        qInfo() << "Populated fixture ready ms" << startup.elapsed() << "requests through readiness" << initialMethods.size() - 1;
        const auto ratio = window.devicePixelRatioF();
        bool hasExpectedDpr = false;
        const auto expectedDpr = qEnvironmentVariable("CGACP_EXPECT_DPR").toDouble(&hasExpectedDpr);
        if (hasExpectedDpr)
            QVERIFY(qAbs(ratio - expectedDpr) < 0.01);
        qInfo() << "Qt platform" << QGuiApplication::platformName() << "devicePixelRatio" << ratio;
        qInfo() << "Screen logical" << window.screen()->geometry() << "available" << window.screen()->availableGeometry()
                << "font" << window.font().family() << window.font().pointSizeF()
                << "initial appearance dark, comfortable, text 100%";
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
        auto *advancedAppearance = window.findChild<QToolButton *>("appearanceAdvancedToggle");
        QVERIFY(advancedAppearance && !advancedAppearance->isChecked());
        QTest::mouseClick(advancedAppearance, Qt::LeftButton);
        QVERIFY(captureWidget(window, "settings-appearance-advanced"));
        QTest::mouseClick(advancedAppearance, Qt::LeftButton);
        auto *density = window.findChild<QSpinBox *>("densityChoice");
        QVERIFY(density);
        chooseTheme(window, "Light");
        QVERIFY(QApplication::palette().color(QPalette::Window).lightness() > 128);
        density->setValue(4);
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
        chooseTheme(window, "Dark");
        checkbox->setFocus();
        QVERIFY(verifyTick());
        QVERIFY(captureWidget(window, "repositories-checked-dark"));
        QTest::keyClick(checkbox, Qt::Key_Space);
        QVERIFY(!checkbox->isChecked());

        settingsSection(window, "about");
        QVERIFY(window.findChild<QLabel *>("aboutDetails")->text().contains("HANDOFF ONLY"));
        QVERIFY(window.findChild<QLabel *>("aboutDetails")->text().contains("ISOLATED EXECUTION · BLOCKED"));
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
        chooseTheme(window, "Light");
        chooseTheme(window, "Dark");
        QTest::keyClick(back, Qt::Key_Space);
        QTRY_VERIFY(composer->isVisible());
        QCOMPARE(composer->toPlainText(), tr("Draft kept while reviewing preferences"));
        composer->setFocus();
        QVERIFY(captureWidget(window, "chat-draft"));
        client.request("test.crash", {}, {});
        QTRY_VERIFY(!client.isRunning());
        QPointer<QWidget> toast = window.findChild<QWidget *>("toast");
        QTRY_VERIFY(toast && toast->isVisible());
        auto *toastBody = toast->findChild<QLabel *>("toastBody");
        QCOMPARE(toastBody->text().simplified(), QString("The runtime stopped. Reconnect to restore saved state."));
        QTRY_VERIFY(toastBody->height() >= toastBody->fontMetrics().height() +
                    toastBody->text().count(u'\n') * toastBody->fontMetrics().lineSpacing());
        const auto *host = toast->parentWidget();
        const QRect editorBounds(composer->mapTo(host, QPoint{}), composer->size());
        QVERIFY(toast->geometry().bottom() < editorBounds.top());
        QVERIFY(toast->geometry().top() <= 100);
        QVERIFY(toast->parentWidget()->width() - toast->geometry().right() <= 20);
        QVERIFY(toast->height() <= 96);
        QCOMPARE(composer->toPlainText(), tr("Draft kept while reviewing preferences"));
        QVERIFY(captureWidget(window, "toast-disconnected"));
        QTest::mouseClick(toast->findChild<QToolButton *>("dismissToast"), Qt::LeftButton);
        QTRY_VERIFY(!toast || !toast->isVisible());
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
        interaction_cases::pinToPrimary(window);
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
        interaction_cases::pinToPrimary(window);
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
        interaction_cases::pinToPrimary(failed);
        failed.show();
        reject.start();
        QTRY_VERIFY(failed.findChild<QPushButton *>("addAutomation")->isEnabled());
        settingsSection(failed, "appearance");
        chooseTheme(failed, "Light");
        QTRY_VERIFY(failed.findChild<QPushButton *>("preferencesRetry")->isVisible());
        QVERIFY(failed.findChild<QLabel *>("preferencesStatus")->text().contains("Could not save"));
        QVERIFY(failed.findChild<QRadioButton *>("themeLight")->isChecked());
        auto *retry = failed.findChild<QPushButton *>("preferencesRetry");
        QTest::keyClick(retry, Qt::Key_Space);
        QTRY_VERIFY(!retry->isVisible());
        QVERIFY(failed.findChild<QRadioButton *>("themeLight")->isChecked());
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
        const auto shown = host.findChildren<QWidget *>("toast");
        QVERIFY(std::any_of(shown.cbegin(), shown.cend(), [](auto *toast) {
            return toast->isVisible() && toast->property("severity").toString() == "error";
        }));
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

    void notificationsStackUpFromClientBottom()
    {
        QWidget host;
        host.resize(600, 330);
        NotificationCenter notices(host);
        host.show();
        notices.post("Earlier notification", "warning");
        notices.post("Newest notification", "error");
        QWidget *older = nullptr;
        QWidget *newest = nullptr;
        for (auto *toast : host.findChildren<QWidget *>("toast")) {
            const auto text = toast->findChild<QLabel *>("toastBody")->text();
            if (text == "Earlier notification")
                older = toast;
            if (text == "Newest notification")
                newest = toast;
        }
        QVERIFY(older && newest);
        QCOMPARE(host.height() - newest->geometry().bottom() - 1, 18);
        QCOMPARE(host.width() - newest->geometry().right() - 1, 18);
        QVERIFY(older->geometry().bottom() < newest->geometry().top());
        host.resize(380, 190);
        QTRY_COMPARE(host.height() - newest->geometry().bottom() - 1, 18);
        QVERIFY(host.rect().contains(newest->geometry()));
        QVERIFY(host.rect().contains(older->geometry()));
    }

    void toastExpiryPausesForHoverAndFocus()
    {
        if (QApplication::platformName() != QStringLiteral("windows"))
            QSKIP("The elapsed-time check runs with native Windows focus.");
        QWidget host;
        host.resize(660, 420);
        auto *editor = new QLineEdit(&host);
        editor->setGeometry(20, 20, 180, 30);
        NotificationCenter notices(host);
        interaction_cases::pinToPrimary(host);
        host.show();
        host.activateWindow();
        QTRY_VERIFY(host.isActiveWindow());
        editor->setFocus();
        QTRY_VERIFY(editor->hasFocus());
        notices.post("Timed information", "info");
        QPointer<QWidget> card = host.findChild<QWidget *>("toast");
        QVERIFY(card && card->isVisible());
        QPointer<QToolButton> dismiss = card->findChild<QToolButton *>("dismissToast");
        QVERIFY(dismiss);
        // Feed Qt's hover state directly so the desktop's pointer cannot move it during the wait.
        card->setAttribute(Qt::WA_UnderMouse);
        QTest::qWait(5600);
        QVERIFY(card && card->isVisible());
        QVERIFY(editor->hasFocus());

        dismiss->setFocus(Qt::TabFocusReason);
        QTRY_VERIFY(dismiss && dismiss->hasFocus());
        card->setAttribute(Qt::WA_UnderMouse, false);
        QTest::qWait(5600);
        QVERIFY(card && card->isVisible());

        editor->setFocus(Qt::TabFocusReason);
        QTRY_VERIFY_WITH_TIMEOUT(!card || !card->isVisible(), 6800);
        QVERIFY(editor->hasFocus());
    }

    void reducedMotionAndLongToast()
    {
        QWidget host;
        host.resize(420, 260);
        auto *history = new QListWidget(&host);
        history->hide();
        NotificationCenter notices(host);
        notices.attachHistory(*history);
        host.show();
        const auto detail = QString("Long notification detail ").repeated(12).trimmed();
        notices.post(detail, "error");
        auto *card = host.findChild<QWidget *>("toast");
        QVERIFY(card && card->graphicsEffect());
        const auto summary = card->findChild<QLabel *>("toastBody")->text();
        QVERIFY(summary.size() < detail.size());
        QVERIFY(summary.count(u'\n') <= 2);
        QVERIFY(history->item(0)->text().contains(detail));
        notices.setReducedMotion(true);
        QVERIFY(!card->graphicsEffect());
        notices.post("Instant warning", "warning");
        for (auto *toast : host.findChildren<QWidget *>("toast"))
            QVERIFY(!toast->graphicsEffect());
    }

    void pagedCatalogUsesBindingId()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "paged"});
        MainWindow window(client);
        window.resize(1366, 768);
        interaction_cases::pinToPrimary(window);
        window.show();
        client.start();
        settingsSection(window, "repositories");
        auto *table = window.findChild<QTableView *>("repositoriesTable");
        auto *load = window.findChild<QPushButton *>("repoLoadMore");
        auto *search = window.findChild<QLineEdit *>("repoSearch");
        QVERIFY(table && load && search);
        QTRY_COMPARE(table->model()->rowCount(), 40);
        table_visual::verifyAlternateRowTail(*table);
        chooseTheme(window, "Light");
        table_visual::verifyAlternateRowTail(*table, QStringLiteral("-light"));
        chooseTheme(window, "Dark");
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

    void archivedCardQueriesDiscardStaleReplies()
    {
        verifyArchivedCardQueries();
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
        interaction_cases::pinToPrimary(window);
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
        verifyRuntimeSettingsData(window, bindingId,
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
        if (auto *settingsBack = window.findChild<QPushButton *>("settingsBack"); settingsBack->isVisible())
            settingsBack->click();
        auto *history = window.findChild<QListWidget *>("conversationList");
        auto *historySearch = window.findChild<QLineEdit *>("conversationSearch");
        QVERIFY(history && historySearch);
        historySearch->setText("Temporary UI task");
        QTRY_VERIFY(history->count() == 1 && history->item(0)->text().contains("Temporary UI task"));
        QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier, history->visualItemRect(history->item(0)).center());
        auto *editor = window.findChild<QPlainTextEdit *>("conversationComposer");
        auto *transcript = window.findChild<QPlainTextEdit *>("conversationTranscript");
        auto *clear = window.findChild<QPushButton *>("clearDraft");
        auto *restore = window.findChild<QPushButton *>("restoreDraft");
        QVERIFY(editor && transcript && clear && restore);
        QTRY_VERIFY(transcript->toPlainText().contains("Check the fixture interface"));
        editor->setPlainText("Unsent fixture draft");
        clear->click();
        QCOMPARE(editor->toPlainText(), QString());
        QVERIFY(restore->isVisible());
        restore->click();
        QCOMPARE(editor->toPlainText(), QStringLiteral("Unsent fixture draft"));
        const auto messageId = accepted.value("message").toObject().value("id");
        const auto taskId = accepted.value("task").toObject().value("id");
        const auto blocked = invoke(client, "message.delete.preview", {{"id", conversationId}, {"message_id", messageId}});
        QVERIFY(blocked.value("blocker_count").toInteger() > 0);
        QCOMPARE(invoke(client, "conversation.cancel", {{"id", conversationId}, {"task_id", taskId}}).value("state").toString(),
                 QStringLiteral("CANCELLED"));
        QTextCursor location(transcript->document());
        location.movePosition(QTextCursor::Down);
        emit transcript->customContextMenuRequested(transcript->cursorRect(location).center());
        auto *menu = qobject_cast<QMenu *>(QApplication::activePopupWidget());
        QVERIFY(menu);
        auto *removeMessage = menu->findChild<QAction *>("deleteSubmittedMessage");
        QVERIFY(removeMessage);
        removeMessage->trigger();
        menu->close();
        QTRY_VERIFY(window.findChild<QMessageBox *>("deleteDataConfirmation"));
        auto *confirmMessage = window.findChild<QMessageBox *>("deleteDataConfirmation");
        QVERIFY(captureWidget(*confirmMessage, "delete-message-confirmation"));
        confirmMessage->button(QMessageBox::Yes)->click();
        QTRY_VERIFY(transcript->toPlainText().isEmpty());
        QTRY_COMPARE(invoke(client, "conversation.get", {{"id", conversationId}}).value("messages").toArray().size(), 0);
        QCOMPARE(editor->toPlainText(), QStringLiteral("Unsent fixture draft"));
        auto *removeConversation = window.findChild<QAction *>("deleteConversation");
        QVERIFY(removeConversation);
        removeConversation->trigger();
        QTRY_VERIFY(window.findChild<QMessageBox *>("deleteDataConfirmation"));
        window.findChild<QMessageBox *>("deleteDataConfirmation")->button(QMessageBox::Yes)->click();
        QTRY_COMPARE(history->count(), 0);
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
        client.start();
        QTRY_COMPARE(connected.size(), 3);
        QCOMPARE(invoke(client, "conversation.get", {{"id", conversationId}}).value("code").toString(),
                 QStringLiteral("NOT_FOUND"));
        historySearch->clear();
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
        QTest::keyClick(window.findChild<QPushButton *>("settingsBack"), Qt::Key_Space);
        auto *expandedSplit = window.findChild<QSplitter *>("shellSplitter");
        auto *handle = expandedSplit->handle(1);
        const auto center = handle->rect().center();
        const int beforeDrag = expandedSplit->sizes().first();
        QTest::mousePress(handle, Qt::LeftButton, Qt::NoModifier, center);
        QTest::mouseMove(handle, center + QPoint(31, 0), 30);
        QTest::mouseRelease(handle, Qt::LeftButton, Qt::NoModifier, center);
        const int savedWidth = expandedSplit->sizes().first();
        QVERIFY(savedWidth > beforeDrag);
        QTest::mouseClick(window.findChild<QToolButton *>("toggleSidebar"), Qt::LeftButton);
        settingsSection(window, "appearance");
        chooseTheme(window, "System");
        auto *motion = window.findChild<QCheckBox *>("reducedMotion");
        QVERIFY(motion);
        motion->setChecked(true);
        QTRY_COMPARE(invoke(client, "preferences.get").value("settings").toObject().value("theme").toString(), QString("system"));
        QTRY_VERIFY(invoke(client, "preferences.get").value("settings").toObject().value("sidebar_collapsed").toBool());
        QTRY_VERIFY(invoke(client, "preferences.get").value("settings").toObject().value("reduced_motion").toBool());
        QCOMPARE(invoke(client, "preferences.get").value("settings").toObject().value("repository_layout")
                     .toObject().value("horizontal_scroll").toInt(), 65);
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
        window.hide();
        RuntimeClient reopenedClient(runtime, {"--stdio", "--data-dir", fixture.filePath("database")});
        MainWindow reopened(reopenedClient);
        reopened.resize(1366, 768);
        interaction_cases::pinToPrimary(reopened);
        reopened.show();
        reopenedClient.start();
        QTRY_VERIFY(reopened.findChild<QRadioButton *>("themeSystem")->isChecked());
        QVERIFY(reopened.findChild<QCheckBox *>("reducedMotion")->isChecked());
        const auto scheme = QGuiApplication::styleHints()->colorScheme();
        if (scheme != Qt::ColorScheme::Unknown)
            QCOMPARE(QApplication::palette().color(QPalette::Window).lightness() > 128, scheme == Qt::ColorScheme::Light);
        if (QApplication::platformName() == QStringLiteral("windows")) {
            struct RestoreScheme {
                ~RestoreScheme() { QGuiApplication::styleHints()->unsetColorScheme(); }
            } restoreScheme;
            QTest::mouseClick(reopened.findChild<QPushButton *>("navAutomations"), Qt::LeftButton);
            auto *add = reopened.findChild<QPushButton *>("addAutomation");
            QTRY_VERIFY(add && add->isVisible());
            QTest::mouseClick(add, Qt::LeftButton);
            auto *dialog = reopened.findChild<QDialog *>("automationEditor");
            QTRY_VERIFY(dialog && dialog->isVisible());
            auto *name = dialog->findChild<QLineEdit *>("automationName");
            QVERIFY(name);
            name->setText(QStringLiteral("Preserved during theme change"));
            for (const auto requested : {Qt::ColorScheme::Light, Qt::ColorScheme::Dark}) {
                QGuiApplication::styleHints()->setColorScheme(requested);
                const bool light = requested == Qt::ColorScheme::Light;
                QTRY_COMPARE(dialog->palette().color(QPalette::Window).lightness() > 128, light);
                QCOMPARE(reopened.palette().color(QPalette::Window).lightness() > 128, light);
                QCOMPARE(name->text(), QStringLiteral("Preserved during theme change"));
            }
            QTest::keyClick(dialog, Qt::Key_Escape);
            QTRY_VERIFY(!reopened.findChild<QDialog *>("automationEditor"));
        }
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
        QTest::keyClick(reopened.findChild<QPushButton *>("settingsBack"), Qt::Key_Space);
        QTest::keyClick(reopened.findChild<QToolButton *>("toggleSidebar"), Qt::Key_Space);
        QCOMPARE(split->sizes().first(), savedWidth);
        settingsSection(reopened, "appearance");
        auto *reset = reopened.findChild<QPushButton *>("resetAppearanceDefault");
        QVERIFY(reset && reset->isVisible());
        QTest::keyClick(reset, Qt::Key_Space);
        QVERIFY(reopened.findChild<QRadioButton *>("themeDark")->isChecked());
        QVERIFY(!reset->isVisible());
        QTest::keyClick(reopened.findChild<QPushButton *>("settingsBack"), Qt::Key_Space);
        QCOMPARE(split->sizes().first(), savedWidth);
        QCOMPARE(reopenedTable->horizontalHeader()->sectionSize(1), 1200);
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

    void chineseCatalogLoadsAfterRuntimePreflight()
    {
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture", "startup-language"});
        const QSignalSpy ready(&client, &RuntimeClient::ready);
        client.start();
        QTRY_COMPARE(ready.size(), 1);
        QCOMPARE(invoke(client, "runtime.info").value("version").toString(), QStringLiteral(CGACP_APP_VERSION));
        const auto settings = invoke(client, "preferences.get").value("settings").toObject();
        QCOMPARE(settings.value("appearance").toObject().value("language").toString(), QStringLiteral("zh_CN"));

        QTranslator translator;
        const auto file = qEnvironmentVariable("CGACP_TEST_ZH_QM",
                                               QFileInfo(qEnvironmentVariable("CGACP_TEST_QM")).dir()
                                                   .filePath("cgacp_zh_CN.qm"));
        QVERIFY(translator.load(file));
        QCOMPARE(translator.translate("MainWindow", "Language"), QStringLiteral("语言"));
        QCOMPARE(translator.translate("StoragePage", "Storage"), QStringLiteral("存储"));
        QCOMPARE(translator.translate("ArchivedPage", "Archived chats"), QStringLiteral("已归档对话"));
        struct RestoreLanguage {
            QTranslator &translator;
            ~RestoreLanguage() { QCoreApplication::removeTranslator(&translator); }
        } restore{translator};
        QCoreApplication::installTranslator(&translator);
        MainWindow window(client, nullptr, true);
        window.show();
        QTRY_VERIFY(window.findChild<QPushButton *>("addAutomation")->isEnabled());
        QCOMPARE(window.findChild<QPushButton *>("navAutomations")->text(), QStringLiteral("自动化"));
        settingsSection(window, "appearance");
        QCOMPARE(window.findChild<QLineEdit *>("settingsSearch")->placeholderText(), QStringLiteral("搜索设置"));
        QCOMPARE(ready.size(), 1);
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
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
        interaction_cases::pinToPrimary(window);
        window.show();
        client.start();
        QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 3);
        auto *nav = window.findChild<QPushButton *>("navAutomations");
        QCOMPARE(nav->text(), QStringLiteral("Tự động hóa"));
        QVERIFY(nav->fontMetrics().horizontalAdvance(nav->text()) + 44 <= nav->width());
        QVERIFY(captureWidget(window, "vi-automations"));
        settingsSection(window, "appearance");
        auto *scaling = window.findChild<QSpinBox *>("textScale");
        auto *advanced = window.findChild<QToolButton *>("appearanceAdvancedToggle");
        QVERIFY(scaling && advanced);
        QVERIFY(!scaling->isVisible());
        QTest::keyClick(advanced, Qt::Key_Space);
        QVERIFY(scaling->isVisible());
        scaling->setValue(16);
        auto *brand = window.findChild<QLabel *>("brand");
        auto *appearance = window.findChild<QWidget *>("appearancePage");
        QVERIFY(brand && appearance);
        QVERIFY(!brand->isVisible());
        for (auto *label : appearance->findChildren<QLabel *>()) {
            if (label->buddy() && label->isVisible())
                QTRY_VERIFY(label->width() >= label->fontMetrics().horizontalAdvance(label->text().remove('&')));
        }
        auto *reset = window.findChild<QPushButton *>("resetAppearanceDefault");
        QTRY_VERIFY(reset && reset->isVisible());
        QTRY_VERIFY(reset->width() >= reset->fontMetrics().horizontalAdvance(reset->text()) + 16);
        QVERIFY(captureWidget(window, "vi-appearance-text16"));
        auto *navigation = window.findChild<QListWidget *>("settingsNavigation");
        QVERIFY(navigation && navigation->currentItem());
        const auto item = navigation->currentItem();
        QVERIFY(navigation->fontMetrics().horizontalAdvance(item->text()) + 16 <=
                navigation->visualItemRect(item).width());
        client.stop();
        QTRY_VERIFY_WITH_TIMEOUT(!client.isRunning(), 6000);
    }

    void mixedDpiWindowMove()
    {
        auto *first = QGuiApplication::primaryScreen();
        QScreen *second = nullptr;
        for (auto *screen : QGuiApplication::screens()) {
            if (qAbs(screen->devicePixelRatio() - first->devicePixelRatio()) > 0.05) {
                second = screen;
                break;
            }
        }
        if (!second)
            QSKIP("This run does not expose two screens with different effective DPI.");
        RuntimeClient client(QCoreApplication::applicationFilePath(), {"--fixture"});
        MainWindow window(client);
        window.resize(QSize(940, 520).boundedTo(first->availableGeometry().size()));
        window.move(first->availableGeometry().topLeft() + QPoint(16, 16));
        interaction_cases::pinToPrimary(window);
        window.show();
        QTRY_COMPARE(window.screen(), first);
        auto *toggle = window.findChild<QToolButton *>("toggleSidebar");
        QTest::mouseClick(toggle, Qt::LeftButton);
        auto *splitter = window.findChild<QSplitter *>("shellSplitter");
        QCOMPARE(splitter->sizes().first(), 60);
        const auto sourceDpr = window.devicePixelRatioF();
        QSignalSpy screenChanges(window.windowHandle(), &QWindow::screenChanged);
        window.move(second->availableGeometry().topLeft() + QPoint(16, 16));
        QTRY_COMPARE(window.screen(), second);
        QTRY_VERIFY(qAbs(window.devicePixelRatioF() - sourceDpr) > 0.05);
        QVERIFY(!screenChanges.isEmpty());
        qInfo() << "Window moved across DPI" << sourceDpr << "to" << window.devicePixelRatioF();
        QCOMPARE(splitter->sizes().first(), 60);
        auto *settings = window.findChild<QToolButton *>("navSettings");
        QVERIFY(settings->isVisible() && !settings->icon().isNull());
        QVERIFY(captureWidget(window, "mixed-dpi-sidebar"));
    }
};

int main(int argc, char **argv)
{
    if (argc >= 2 && QByteArray(argv[1]) == "--fixture")
        return runFixture(argc == 3 ? QString::fromLatin1(argv[2]) : QString{});
    QApplication::setAttribute(Qt::AA_DontUseNativeDialogs);
    QApplication app(argc, argv);
    auto fontPath = qEnvironmentVariable("CGACP_TEST_FONT");
#ifdef Q_OS_WIN
    if (fontPath.isEmpty() && QGuiApplication::platformName() == "offscreen") {
        const auto windows = qEnvironmentVariable("WINDIR");
        const auto candidate = QDir(windows).filePath(QStringLiteral("Fonts/segoeui.ttf"));
        if (!windows.isEmpty() && QFileInfo(candidate).isFile())
            fontPath = candidate;
    }
#endif
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
