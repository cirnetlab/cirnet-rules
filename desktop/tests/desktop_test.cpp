#include "build_info.h"
#include "main_window.h"
#include "runtime_client.h"

#include <QAbstractItemModel>
#include <QApplication>
#include <QCoreApplication>
#include <QDialogButtonBox>
#include <QDir>
#include <QEventLoop>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QFont>
#include <QFontDatabase>
#include <QIcon>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QPushButton>
#include <QSignalSpy>
#include <QStandardPaths>
#include <QTest>
#include <QTemporaryDir>
#include <QTableView>
#include <QTimer>
#include <QTranslator>
#include <cstdio>

namespace {
QJsonObject fixtureResult(const QString &method)
{
    if (method == "runtime.info")
        return {{"product_generation", QStringLiteral(CGACP_PRODUCT_GENERATION)},
                {"version", QStringLiteral(CGACP_APP_VERSION)},
                {"release_name", QStringLiteral(CGACP_RELEASE_NAME)}};
    if (method == "repo.list")
        return {{"items", QJsonArray{QJsonObject{{"display_name", "Fixture UI"},
                 {"root", "workspace/Fixture UI"}, {"repo_id", "repo-fixture"},
                 {"binding_id", "binding-fixture"}, {"status", "AVAILABLE"}, {"revision", 1}}}}};
    if (method == "automation.list")
        return {{"items", QJsonArray{
                    QJsonObject{{"id", "read-check"}, {"name", "Scoped review"}, {"prompt", "Review the selected worktree."},
                                {"binding_id", "binding-fixture"}, {"enabled", true}, {"revision", 1},
                                {"trigger", QJsonObject{{"kind", "DAILY"}, {"hour", 9}, {"minute", 0}, {"timezone", "UTC"}}}},
                    QJsonObject{{"id", "inactive-check"}, {"name", "Review later"}, {"prompt", "Inspect local files."},
                                {"enabled", false}, {"revision", 1}}
                }}};
    if (method == "automation.templates")
        return {{"items", QJsonArray{QJsonObject{{"id", "review"}, {"title", "Scoped code review"},
                {"description", "Inspect selected files."}, {"status", "UNVERIFIED"}, {"revision", 1}}}}};
    return {{"items", QJsonArray{}}};
}

int runFixture()
{
    char line[65536];
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
        const auto response = QJsonDocument(QJsonObject{{"v", 1}, {"id", request.value("id")},
                                  {"ok", true}, {"result", fixtureResult(method)}})
                              .toJson(QJsonDocument::Compact) + '\n';
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
        const auto *brandMark = window.findChild<QLabel *>("brandMark");
        QVERIFY(brandMark);
        QVERIFY(!brandMark->pixmap().isNull());
        QVERIFY(!QApplication::windowIcon().isNull());
        const auto scale = window.devicePixelRatioF();
        const auto resizeForDisplay = [&](int width, int height) {
            window.resize(qRound(width / scale), qRound(height / scale));
        };
        resizeForDisplay(1366, 768);
        window.show();
        client.start();
        auto *automations = window.findChild<QPushButton *>("navAutomations");
        auto *repositories = window.findChild<QPushButton *>("navRepositories");
        QVERIFY(automations);
        QVERIFY(repositories);
        QTest::keyClick(automations, Qt::Key_Space);
        QTRY_VERIFY(window.findChild<QLabel *>("automationCardTitle"));
        QCOMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 2);
        const auto small = window.grab();
        QVERIFY(qAbs(small.width() - 1366) <= 2);
        QVERIFY(qAbs(small.height() - 768) <= 2);
        const auto capture = qEnvironmentVariable("CGACP_UI_CAPTURE");
        if (!capture.isEmpty()) {
            QVERIFY(small.save(capture + "/cgacp-1366x768.png"));
            resizeForDisplay(1920, 1080);
            QTest::qWait(60);
            const auto large = window.grab();
            QVERIFY(qAbs(large.width() - 1920) <= 2);
            QVERIFY(qAbs(large.height() - 1080) <= 2);
            QVERIFY(large.save(capture + "/cgacp-1920x1080.png"));
        }
        QTest::keyClick(repositories, Qt::Key_Space);
        QTRY_VERIFY(window.findChild<QPushButton *>("selectRepositoryFolder")->isVisible());
        QTRY_VERIFY(window.findChild<QPushButton *>("selectRepositoryFile")->isVisible());
        client.stop();
        QTRY_VERIFY(!client.isRunning());
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

        auto *navigation = window.findChild<QPushButton *>("navRepositories");
        QVERIFY(navigation);
        QTest::keyClick(navigation, Qt::Key_Space);
        auto *picker = window.findChild<QPushButton *>("selectRepositoryFile");
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
        QTest::keyClick(picker, Qt::Key_Space);
        QTRY_VERIFY(confirmed);
        drivePicker.stop();
        auto *table = window.findChild<QTableView *>("repositoriesTable");
        QVERIFY(table);
        QTRY_COMPARE(table->model()->rowCount(), 1);
        const auto binding = invoke(client, "repo.list").value("items").toArray().first().toObject();
        const auto bindingId = binding.value("binding_id").toString();
        QVERIFY(!bindingId.isEmpty());
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
        QTRY_VERIFY(!client.isRunning());
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
            {"revision", binding.value("revision")}, {"path", relocated}, {"token", relocation.value("token")}});
        QCOMPARE(relink.value("binding_id").toString(), bindingId);
        QCOMPARE(relink.value("revision").toInteger(), binding.value("revision").toInteger() + 1);
        const auto capture = qEnvironmentVariable("CGACP_UI_CAPTURE");
        if (!capture.isEmpty()) {
            QTest::keyClick(navigation, Qt::Key_Space);
            QTRY_COMPARE(table->model()->rowCount(), 1);
            QVERIFY(window.grab().save(capture + "/cgacp-real-runtime-repositories.png"));
        }
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }

    void vietnameseCatalogLoads()
    {
        QTranslator translator;
        const auto file = qEnvironmentVariable("CGACP_TEST_QM");
        QVERIFY(!file.isEmpty());
        QVERIFY(translator.load(file));
        QCOMPARE(translator.translate("MainWindow", "Automations"), QStringLiteral("Tự động hóa"));
        QCOMPARE(translator.translate("MainWindow", "Open from file"), QStringLiteral("Chọn tệp trong repo"));
        QCOMPARE(translator.translate("MainWindow", "Cancel pending task"), QStringLiteral("Hủy nhiệm vụ đang chờ"));
    }
};

int main(int argc, char **argv)
{
    if (argc == 2 && QByteArray(argv[1]) == "--fixture")
        return runFixture();
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
