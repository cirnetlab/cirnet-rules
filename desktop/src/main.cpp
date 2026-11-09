#include "build_info.h"
#include "live_text.h"
#include "main_window.h"
#include "runtime_client.h"

#include <QApplication>
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QTimer>
#include <memory>

int main(int argc, char **argv)
{
    QApplication app(argc, argv);
    QApplication::setApplicationName(QStringLiteral("cgacp-dashboard"));
    QApplication::setApplicationVersion(QStringLiteral(CGACP_APP_VERSION));
    QApplication::setOrganizationName(QStringLiteral("CIRNETLAB"));
    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("CIRNET Governance & Agent Control Plane"));
    parser.addHelpOption();
    parser.addVersionOption();
    QCommandLineOption dataDir(QStringLiteral("data-dir"), QStringLiteral("Local runtime data directory"),
                               QStringLiteral("path"));
    QCommandLineOption language(QStringLiteral("language"), QStringLiteral("Dashboard language (en, vi or zh-CN)"),
                                QStringLiteral("locale"));
    parser.addOptions({dataDir, language});
    parser.process(app);
    applyTheme(app);
    QStringList arguments{QStringLiteral("--stdio")};
    if (parser.isSet(dataDir))
        arguments << QStringLiteral("--data-dir") << parser.value(dataDir);
    RuntimeClient runtime(RuntimeClient::siblingExecutable(), arguments);
    std::unique_ptr<MainWindow> window;
    QObject bootstrap;
    QTimer languageDeadline;
    languageDeadline.setSingleShot(true);
    bool ready = false;
    const auto showWindow = [&](const QString &requested, const QString &startupError = {}) {
        if (window)
            return;
        languageDeadline.stop();
        QObject::disconnect(&runtime, nullptr, &bootstrap, nullptr);
        live::language().set(requested);
        window = std::make_unique<MainWindow>(runtime, nullptr, ready, startupError, !parser.isSet(language));
        window->show();
    };
    if (parser.isSet(language)) {
        showWindow(parser.value(language));
    } else {
        // Keep the single runtime connection and bound the wait before showing an English fallback.
        QObject::connect(&runtime, &RuntimeClient::ready, &bootstrap, [&] {
            ready = true;
            runtime.request("runtime.info", {}, [&](const QJsonObject &info, const QString &error) {
                if (window || !error.isEmpty())
                    return;
                if (info.value("version").toString() != QStringLiteral(CGACP_APP_VERSION)) {
                    showWindow({});
                    return;
                }
                runtime.request("preferences.get", {}, [&](const QJsonObject &result, const QString &failure) {
                    if (window || !failure.isEmpty())
                        return;
                    showWindow(result.value("settings").toObject().value("appearance").toObject()
                                         .value("language").toString());
                });
            });
        });
        QObject::connect(&runtime, &RuntimeClient::disconnected, &bootstrap, [&](const QString &error) {
            ready = false;
            showWindow({}, error);
        });
        QObject::connect(&languageDeadline, &QTimer::timeout, &bootstrap, [&] { showWindow({}); });
        languageDeadline.start(2500);
    }
    runtime.start();
    return app.exec();
}
