#include "build_info.h"
#include "live_text.h"
#include "main_window.h"
#include "runtime_client.h"

#include <QApplication>
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QJsonDocument>
#include <QMessageBox>
#include <QProgressDialog>
#include <QPushButton>
#include <QThread>
#include <QTimer>
#include <functional>
#include <memory>

namespace {
class IdentityUpgrade {
    Q_DECLARE_TR_FUNCTIONS(IdentityUpgrade)
public:
    using Done = std::function<void(const QJsonObject &, const QString &)>;

    static void run(QObject &context, QString mode, const QStringList &runtimeArgs, Done done)
    {
        auto args = runtimeArgs;
        args[0] = mode;
        const auto program = RuntimeClient::siblingExecutable();
        struct Outcome { QJsonObject result; QString error; };
        const auto outcome = std::make_shared<Outcome>();
        auto *worker = QThread::create([program, args, mode, outcome] {
            outcome->error = RuntimeClient::validateExecutable(program, true);
            if (!outcome->error.isEmpty())
                return;
            QProcess process;
            process.setStandardErrorFile(QProcess::nullDevice());
            process.start(program, args);
            if (!process.waitForStarted(3000) ||
                (process.state() != QProcess::NotRunning &&
                 !process.waitForFinished(mode == "--identity-preview" ? 5000 : 90000))) {
                process.kill();
                process.waitForFinished(3000);
                outcome->error = tr("The local data check could not finish. Reopen OPENRULES to retry.");
                return;
            }
            const auto response = process.readAllStandardOutput();
            if (process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0 ||
                response.size() > 64 * 1024) {
                outcome->error = tr("The runtime could not inspect the local data.");
                return;
            }
            outcome->result = QJsonDocument::fromJson(response).object();
            if (outcome->result.value("status") == QStringLiteral("ERROR")) {
                const auto issue = outcome->result.value("error").toObject();
                if (issue.value("code") == QStringLiteral("STORAGE_BUSY"))
                    outcome->error = tr("Local data is in use. Close the previous Desktop and its runtime, then retry.");
                else
                    outcome->error = issue.value("message").toString();
            }
            if (outcome->result.value("status").toString().isEmpty())
                outcome->error = tr("The runtime sent an invalid data-upgrade response.");
        });
        QObject::connect(worker, &QThread::finished, &context, [outcome, done = std::move(done)] {
            done(outcome->result, outcome->error);
        });
        QObject::connect(worker, &QThread::finished, worker, &QObject::deleteLater);
        worker->start();
    }
};
}

int main(int argc, char **argv)
{
    QApplication app(argc, argv);
    QApplication::setApplicationName(QStringLiteral("openrules-desktop"));
    QApplication::setApplicationDisplayName(QStringLiteral(OPENRULES_PRODUCT_NAME));
    QApplication::setApplicationVersion(QStringLiteral(OPENRULES_APP_VERSION));
    QApplication::setOrganizationName(QStringLiteral(OPENRULES_PRODUCT_NAME));
    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("OPENRULES"));
    parser.addHelpOption();
    parser.addVersionOption();
    QCommandLineOption dataDir(QStringLiteral("data-dir"), QStringLiteral("Local runtime data directory"),
                               QStringLiteral("path"));
    QCommandLineOption language(QStringLiteral("language"), QStringLiteral("Dashboard language (en, vi or zh-CN)"),
                                QStringLiteral("locale"));
    parser.addOptions({dataDir, language});
    parser.process(app);
    applyTheme(app);
    app.setQuitOnLastWindowClosed(false);
    if (parser.isSet(language))
        live::language().set(parser.value(language));
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
        app.setQuitOnLastWindowClosed(true);
    };
    const auto startDesktop = [&] {
        if (parser.isSet(language)) {
            showWindow(parser.value(language));
            runtime.start();
            return;
        }
        // Keep the single runtime connection and bound the wait before showing an English fallback.
        QObject::connect(&runtime, &RuntimeClient::ready, &bootstrap, [&] {
            ready = true;
            runtime.request("runtime.info", {}, [&](const QJsonObject &info, const QString &error) {
                if (window || !error.isEmpty())
                    return;
                if (info.value("version").toString() != QStringLiteral(OPENRULES_APP_VERSION)) {
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
        runtime.start();
    };
    std::unique_ptr<QProgressDialog> progress;
    const auto title = IdentityUpgrade::tr("Local data upgrade");
    const auto reportError = [&](const QString &message) {
        QMessageBox::critical(nullptr, title, message);
        app.quit();
    };
    IdentityUpgrade::run(app, "--identity-preview", arguments, [&](const QJsonObject &result, const QString &error) {
        if (!error.isEmpty()) {
            if (result.isEmpty())
                showWindow(parser.value(language), error);
            else
                reportError(error);
            return;
        }
        const auto state = result.value("status").toString();
        if (state == "READY") {
            startDesktop();
            return;
        }
        const bool conflict = state == "CONFLICT";
        if (!conflict && state != "MIGRATION_AVAILABLE" && state != "RESUME") {
            reportError(IdentityUpgrade::tr("The runtime sent an invalid data-upgrade response."));
            return;
        }
        QMessageBox dialog(QMessageBox::Information, title,
            conflict
                ? IdentityUpgrade::tr("The current and previous databases both contain local data. Choose Use current data to keep them separate, or close OPENRULES.")
                : IdentityUpgrade::tr("Upgrade the previous Desktop data in place of a fresh database. Close the previous Desktop and its runtime before continuing."),
            QMessageBox::Cancel);
        dialog.setTextFormat(Qt::PlainText);
        dialog.setInformativeText(IdentityUpgrade::tr("Previous database\n%1\n\nOPENRULES database\n%2")
            .arg(result.value("source").toString(), result.value("destination").toString())
            + (conflict ? QString{} : QStringLiteral("\n\n")
                + IdentityUpgrade::tr("Keep the old runtime closed after the upgrade. The old database is reserved for offline recovery. Later writes from the old runtime would diverge.")));
        auto *proceed = dialog.addButton(conflict ? IdentityUpgrade::tr("Use current data") :
            IdentityUpgrade::tr("Back up and upgrade"), QMessageBox::AcceptRole);
        dialog.setDefaultButton(QMessageBox::Cancel);
        dialog.exec();
        if (dialog.clickedButton() != proceed) {
            app.quit();
            return;
        }
        progress = std::make_unique<QProgressDialog>();
        progress->setWindowTitle(title);
        progress->setLabelText(IdentityUpgrade::tr("Checking local data…"));
        progress->setRange(0, 0);
        progress->setCancelButton(nullptr);
        progress->setWindowFlag(Qt::WindowCloseButtonHint, false);
        progress->setWindowModality(Qt::ApplicationModal);
        progress->show();
        IdentityUpgrade::run(app, conflict ? "--identity-use-current" : "--identity-migrate", arguments,
            [&](const QJsonObject &response, const QString &failure) {
                progress.reset();
                if (!failure.isEmpty())
                    reportError(failure);
                else if (response.value("status") == "READY")
                    startDesktop();
                else
                    reportError(IdentityUpgrade::tr("Local data still requires recovery. Reopen OPENRULES to retry."));
            });
    });
    return app.exec();
}
