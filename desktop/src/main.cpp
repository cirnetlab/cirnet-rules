#include "build_info.h"
#include "live_text.h"
#include "main_window.h"
#include "runtime_client.h"

#include <QApplication>
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QJsonArray>
#include <QJsonDocument>
#include <QDeadlineTimer>
#include <QPointer>
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

    static void run(QObject &context, QString mode, const QStringList &runtimeArgs, Done done,
                    std::shared_ptr<std::atomic_bool> cancelled = {},
                    std::function<void(QString)> progress = {})
    {
        auto args = runtimeArgs;
        args[0] = mode;
        const auto program = RuntimeClient::siblingExecutable();
        struct Outcome { QJsonObject result; QString error; };
        const auto outcome = std::make_shared<Outcome>();
        auto *worker = QThread::create([program, args, mode, outcome, cancelled, progress, &context] {
            outcome->error = RuntimeClient::validateExecutable(program, true);
            if (!outcome->error.isEmpty())
                return;
            QProcess process;
            process.start(program, args);
            if (!process.waitForStarted(3000)) {
                outcome->error = tr("The runtime could not start.");
                return;
            }
            QDeadlineTimer deadline(mode == "--identity-preview" ? 5000 : 90000);
            bool cancelSent = false;
            while (process.state() != QProcess::NotRunning && !deadline.hasExpired()) {
                if (!cancelSent && cancelled && cancelled->load(std::memory_order_relaxed)) {
                    process.write("cancel\n");
                    cancelSent = true;
                }
                process.waitForFinished(80);
                process.setReadChannel(QProcess::StandardError);
                while (process.canReadLine()) {
                    const auto line = process.readLine(256).trimmed();
                    if (progress && line.startsWith("identity:")) {
                        const auto phase = QString::fromUtf8(line.sliced(9));
                        QMetaObject::invokeMethod(&context, [progress, phase] { progress(phase); }, Qt::QueuedConnection);
                    }
                }
                process.setReadChannel(QProcess::StandardOutput);
            }
            if (process.state() != QProcess::NotRunning) {
                process.kill();
                process.waitForFinished(3000);
                outcome->error = tr("The local data check could not finish. Reopen %1 to retry.")
                                     .arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME));
                return;
            }
            const auto response = process.readAllStandardOutput();
            if (process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0 ||
                response.size() > 64 * 1024) {
                outcome->error = tr("The runtime could not inspect the local data.");
                return;
            }
            outcome->result = QJsonDocument::fromJson(response).object();
            if (outcome->result.value("product_id") != QStringLiteral(CIRNETRULES_PRODUCT_ID)
                || outcome->result.value("protocol") != CIRNETRULES_IPC_PROTOCOL) {
                outcome->result = {};
                outcome->error = tr("The data-upgrade runtime is incompatible. Use the runtime installed with %1.")
                                     .arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME));
                return;
            }
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
    QApplication::setApplicationName(QStringLiteral(CIRNETRULES_PRODUCT_ID));
    QApplication::setApplicationDisplayName(QStringLiteral(CIRNETRULES_PRODUCT_NAME));
    QApplication::setApplicationVersion(QStringLiteral(CIRNETRULES_APP_VERSION));
    QApplication::setOrganizationName(QStringLiteral(CIRNETRULES_ORGANIZATION));
    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral(CIRNETRULES_PRODUCT_NAME));
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
            runtime.request("preferences.get", {}, [&](const QJsonObject &result, const QString &failure) {
                if (window || !failure.isEmpty())
                    return;
                showWindow(result.value("settings").toObject().value("appearance").toObject()
                                     .value("language").toString());
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
        const bool canUseCurrent = result.value("can_use_current").toBool();
        QMessageBox dialog(QMessageBox::Information, title,
            conflict
                ? IdentityUpgrade::tr("Independent data stores were found. Choose an available current store or close Desktop to review the source paths.")
                : IdentityUpgrade::tr("Upgrade the previous Desktop data in place of a fresh database. Close the previous Desktop and its runtime before continuing."),
            QMessageBox::Cancel);
        dialog.setTextFormat(Qt::PlainText);
        QStringList locations;
        for (const auto &candidate : result.value("candidates").toArray()) {
            const auto entry = candidate.toObject();
            locations.append(IdentityUpgrade::tr("%1 database (%2 bytes, WAL %3 bytes)\n%4")
                .arg(entry.value("product").toString()).arg(entry.value("database_bytes").toInteger())
                .arg(entry.value("wal_bytes").toInteger()).arg(entry.value("path").toString()));
        }
        locations.append(IdentityUpgrade::tr("%1 database\n%2")
            .arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME), result.value("destination").toString()));
        if (!conflict)
            locations.append(IdentityUpgrade::tr("Keep the old runtime closed after the upgrade. The old database is reserved for offline recovery. Later writes from the old runtime would diverge."));
        dialog.setInformativeText(locations.join(QStringLiteral("\n\n")));
        QPushButton *proceed = nullptr;
        if (!conflict || canUseCurrent)
            proceed = dialog.addButton(conflict ? IdentityUpgrade::tr("Use current data") :
                IdentityUpgrade::tr("Back up and upgrade"), QMessageBox::AcceptRole);
        dialog.setDefaultButton(QMessageBox::Cancel);
        dialog.exec();
        if (!proceed || dialog.clickedButton() != proceed) {
            app.quit();
            return;
        }
        progress = std::make_unique<QProgressDialog>();
        progress->setWindowTitle(title);
        progress->setLabelText(IdentityUpgrade::tr("Checking local data…"));
        progress->setRange(0, 0);
        auto cancelled = std::make_shared<std::atomic_bool>(false);
        progress->setCancelButtonText(IdentityUpgrade::tr("Pause upgrade"));
        QObject::connect(progress.get(), &QProgressDialog::canceled, &app,
                         [cancelled] { cancelled->store(true, std::memory_order_relaxed); });
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
                    reportError(IdentityUpgrade::tr("Local data still requires recovery. Reopen %1 to retry.")
                        .arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME)));
            }, cancelled, [bar = QPointer<QProgressDialog>(progress.get())](const QString &phase) {
                if (!bar || bar->wasCanceled())
                    return;
                if (phase == "BACKUP")
                    bar->setLabelText(IdentityUpgrade::tr("Backing up local data…"));
                else if (phase == "VERIFY")
                    bar->setLabelText(IdentityUpgrade::tr("Verifying the candidate database…"));
                else if (phase == "ACTIVATE") {
                    bar->setCancelButton(nullptr);
                    bar->setLabelText(IdentityUpgrade::tr("Activating the verified database…"));
                }
            });
    });
    return app.exec();
}
