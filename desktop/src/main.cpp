#include "build_info.h"
#include "main_window.h"
#include "runtime_client.h"

#include <QApplication>
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QLocale>
#include <QTranslator>

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
    QCommandLineOption language(QStringLiteral("language"), QStringLiteral("Dashboard language (en or vi)"),
                                QStringLiteral("locale"));
    parser.addOptions({dataDir, language});
    parser.process(app);
    QTranslator translator;
    const auto locale = parser.value(language);
    if ((locale == QStringLiteral("vi") || (locale.isEmpty() && QLocale::system().language() == QLocale::Vietnamese)) &&
        translator.load(QStringLiteral(":/i18n/cgacp_vi.qm"))) {
        app.installTranslator(&translator);
    }
    applyTheme(app);
    QStringList arguments{QStringLiteral("--stdio")};
    if (parser.isSet(dataDir))
        arguments << QStringLiteral("--data-dir") << parser.value(dataDir);
    RuntimeClient runtime(RuntimeClient::siblingExecutable(), arguments);
    MainWindow window(runtime);
    window.show();
    runtime.start();
    return app.exec();
}
