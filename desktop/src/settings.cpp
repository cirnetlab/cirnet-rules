#include "main_window.h"
#include "build_info.h"
#include "notification_center.h"

#include <QApplication>
#include <QClipboard>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollArea>
#include <QSignalBlocker>
#include <QSpinBox>
#include <QStackedWidget>
#include <QTimer>
#include <QVBoxLayout>

namespace {
QLabel *label(const QString &text, const QString &role)
{
    auto *widget = new QLabel(text);
    widget->setObjectName(role);
    widget->setTextFormat(Qt::PlainText);
    widget->setWordWrap(true);
    return widget;
}

QWidget *scrollPanel(QWidget *body)
{
    auto *scroll = new QScrollArea;
    scroll->setWidgetResizable(true);
    scroll->setWidget(body);
    scroll->setMinimumWidth(0);
    return scroll;
}

void selectData(QComboBox *combo, const QString &data)
{
    if (const int index = combo->findData(data); index >= 0)
        combo->setCurrentIndex(index);
}
}

QWidget *MainWindow::makeSettings()
{
    auto *page = new QWidget;
    page->setObjectName("settingsPage");
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(20, 14, 20, 16);
    layout->setSpacing(10);
    auto *header = new QHBoxLayout;
    auto *back = new QPushButton(tr("Back"));
    back->setObjectName("settingsBack");
    connect(back, &QPushButton::clicked, this, &MainWindow::closeSettings);
    header->addWidget(back);
    header->addWidget(label(tr("Settings"), "heading"), 1);
    settingsChoice_ = new QComboBox;
    settingsChoice_->setObjectName("settingsSection");
    settingsChoice_->setAccessibleName(tr("Settings section"));
    header->addWidget(settingsChoice_);
    layout->addLayout(header);

    settingsPages_ = new QStackedWidget;
    settingsPages_->setObjectName("settingsPages");
    layout->addWidget(settingsPages_, 1);
    const auto section = [this](const QString &title, const QString &key, QWidget *panel) {
        settingsChoice_->addItem(title, key);
        settingsPages_->addWidget(panel);
    };

    auto *appearance = new QWidget;
    appearance->setObjectName("appearancePage");
    auto *appearanceLayout = new QVBoxLayout(appearance);
    appearanceLayout->setContentsMargins(2, 8, 2, 8);
    appearanceLayout->addWidget(label(tr("Appearance"), "heading"));
    appearanceLayout->addWidget(label(tr("Customize the desktop layout, theme and text size."), "lead"));
    auto *appearanceForm = new QFormLayout;
    appearanceForm->setRowWrapPolicy(QFormLayout::WrapLongRows);
    appearanceForm->setFieldGrowthPolicy(QFormLayout::FieldsStayAtSizeHint);
    themeChoice_ = new QComboBox;
    themeChoice_->setObjectName("themeChoice");
    themeChoice_->addItem(tr("Dark"), "dark");
    themeChoice_->addItem(tr("Light"), "light");
    themeChoice_->addItem(tr("System"), "system");
    densityChoice_ = new QComboBox;
    densityChoice_->setObjectName("densityChoice");
    densityChoice_->addItem(tr("Comfortable"), "comfortable");
    densityChoice_->addItem(tr("Compact"), "compact");
    textScale_ = new QSpinBox;
    textScale_->setObjectName("textScale");
    textScale_->setRange(85, 150);
    textScale_->setSingleStep(5);
    textScale_->setValue(100);
    textScale_->setSuffix(QStringLiteral("%"));
    appearanceForm->addRow(tr("&Theme"), themeChoice_);
    appearanceForm->addRow(tr("&Density"), densityChoice_);
    appearanceForm->addRow(tr("Text &size"), textScale_);
    appearanceLayout->addLayout(appearanceForm);
    auto *reset = new QPushButton(tr("Reset layout"));
    reset->setObjectName("resetLayout");
    appearanceLayout->addWidget(reset, 0, Qt::AlignLeft);
    connect(reset, &QPushButton::clicked, this, [this] {
        applyingPreferences_ = true;
        sidebarWidth_ = 260;
        changeSidebar(false);
        restoreRepositoryLayout({});
        applyingPreferences_ = false;
        schedulePreferencesSave();
    });
    appearanceLayout->addStretch();
    section(tr("Appearance"), "appearance", scrollPanel(appearance));
    section(tr("Repositories"), "repositories", repositoriesPage_);

    auto *runtime = new QWidget;
    auto *runtimeLayout = new QVBoxLayout(runtime);
    runtimeLayout->setContentsMargins(2, 8, 2, 8);
    runtimeLayout->addWidget(label(tr("Agents & Runtime"), "heading"));
    runtimeStatus_ = label({}, "runtimeState");
    runtimeCapabilities_ = label({}, "runtimeCapabilities");
    runtimeLayout->addWidget(runtimeStatus_);
    runtimeLayout->addWidget(runtimeCapabilities_);
    auto *runtimeActions = new QHBoxLayout;
    reconnect_ = new QPushButton(tr("Reconnect"));
    reconnect_->setObjectName("reconnectRuntime");
    runtimeActions->addWidget(reconnect_);
    connect(reconnect_, &QPushButton::clicked, this, &MainWindow::reconnectRuntime);
    auto *diagnostics = new QPushButton(tr("Diagnostics"));
    diagnostics->setObjectName("runtimeDiagnostics");
    runtimeActions->addWidget(diagnostics);
    runtimeActions->addStretch();
    runtimeLayout->addLayout(runtimeActions);
    runtimeLayout->addStretch();
    connect(diagnostics, &QPushButton::clicked, this, [this] {
        QDialog dialog(this);
        dialog.setWindowTitle(tr("Runtime diagnostics"));
        auto *body = new QVBoxLayout(&dialog);
        body->addWidget(label(tr("Review connection and capability information before copying."), "lead"));
        auto *text = new QPlainTextEdit;
        text->setReadOnly(true);
        const auto state = online_ ? tr("Connected") : connecting_ ? tr("Connecting") : tr("Disconnected");
        text->setPlainText(tr("CGACP %1\nRuntime: %2\n%3")
                              .arg(QStringLiteral(CGACP_APP_VERSION), state, runtimeCapabilities_->text()));
        body->addWidget(text);
        auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close);
        auto *copy = buttons->addButton(tr("Copy diagnostics"), QDialogButtonBox::ActionRole);
        connect(copy, &QPushButton::clicked, &dialog, [text] {
            QApplication::clipboard()->setText(text->toPlainText());
        });
        connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        body->addWidget(buttons);
        dialog.resize(480, 300);
        dialog.exec();
    });
    section(tr("Agents & Runtime"), "runtime", scrollPanel(runtime));

    auto *notices = new QWidget;
    auto *noticesLayout = new QVBoxLayout(notices);
    noticesLayout->setContentsMargins(2, 8, 2, 8);
    noticesLayout->addWidget(label(tr("Notifications"), "heading"));
    noticesLayout->addWidget(label(tr("Warnings and errors remain visible. Review notifications from this session below."), "lead"));
    auto *noticeControls = new QHBoxLayout;
    notificationChoice_ = new QComboBox;
    notificationChoice_->setObjectName("notificationLevel");
    notificationChoice_->setAccessibleName(tr("Toast level"));
    notificationChoice_->addItem(tr("All toasts"), "all");
    notificationChoice_->addItem(tr("Warnings and errors"), "important");
    noticeControls->addWidget(notificationChoice_);
    auto *clear = new QPushButton(tr("Clear read"));
    clear->setObjectName("clearNotifications");
    noticeControls->addWidget(clear);
    noticeControls->addStretch();
    noticesLayout->addLayout(noticeControls);
    auto *history = new QListWidget;
    history->setObjectName("notificationHistory");
    history->setAccessibleName(tr("Notification history"));
    history->setWordWrap(true);
    noticesLayout->addWidget(history, 1);
    notifications_->attachHistory(*history);
    connect(clear, &QPushButton::clicked, notifications_, &NotificationCenter::clearRead);
    section(tr("Notifications"), "notifications", notices);

    auto *about = new QWidget;
    about->setObjectName("aboutPage");
    auto *aboutLayout = new QVBoxLayout(about);
    aboutLayout->setContentsMargins(2, 8, 2, 8);
    aboutLayout->addWidget(label(tr("About CGACP"), "heading"));
    aboutLayout->addWidget(label(tr("CIRNET Governance & Agent Control Plane"), "lead"));
    aboutDetails_ = label({}, "aboutDetails");
    aboutDetails_->setTextInteractionFlags(Qt::TextSelectableByMouse);
    aboutLayout->addWidget(aboutDetails_);
    aboutLayout->addStretch();
    section(tr("About"), "about", scrollPanel(about));

    auto *saveRow = new QHBoxLayout;
    preferencesStatus_ = label({}, "preferencesStatus");
    preferencesStatus_->hide();
    saveRow->addWidget(preferencesStatus_, 1);
    preferencesRetry_ = new QPushButton(tr("Retry saving"));
    preferencesRetry_->setObjectName("preferencesRetry");
    preferencesRetry_->hide();
    saveRow->addWidget(preferencesRetry_);
    layout->addLayout(saveRow);
    connect(preferencesRetry_, &QPushButton::clicked, this, [this] {
        preferencesRetryRequired_ = false;
        loadPreferences();
    });
    connect(settingsChoice_, &QComboBox::currentIndexChanged, this, [this](int index) {
        settingsPages_->setCurrentIndex(index);
        if (settingsChoice_->currentData().toString() == "notifications")
            notifications_->markHistoryRead();
        else if (settingsChoice_->currentData().toString() == "repositories")
            QTimer::singleShot(0, this, &MainWindow::restoreRepositoryScroll);
    });
    const auto visualChange = [this] {
        theme_->apply(themeChoice_->currentData().toString(), densityChoice_->currentData().toString(),
                      textScale_->value());
        notifications_->setLevel(notificationChoice_->currentData().toString());
        schedulePreferencesSave();
    };
    connect(themeChoice_, &QComboBox::currentIndexChanged, this, visualChange);
    connect(densityChoice_, &QComboBox::currentIndexChanged, this, visualChange);
    connect(textScale_, &QSpinBox::valueChanged, this, visualChange);
    connect(notificationChoice_, &QComboBox::currentIndexChanged, this, visualChange);
    return page;
}

void MainWindow::showSettings(const QString &section)
{
    if (pages_->currentWidget() != settingsPage_)
        previousPage_ = pages_->currentWidget();
    if (!section.isEmpty())
        selectData(settingsChoice_, section);
    showPage(settingsPage_);
    if (settingsChoice_->currentData().toString() == "notifications")
        notifications_->markHistoryRead();
    else if (settingsChoice_->currentData().toString() == "repositories")
        QTimer::singleShot(0, this, &MainWindow::restoreRepositoryScroll);
}

void MainWindow::closeSettings()
{
    if (pages_->currentWidget() == settingsPage_)
        showPage(previousPage_ ? previousPage_ : automationsPage_);
}

void MainWindow::updateRuntimeDetails()
{
    runtimeStatus_->setText(connectionStatus_);
    reconnect_->setEnabled(!online_ && !client_.isRunning());
    preferencesRetry_->setEnabled(online_);
    QStringList adapters;
    const auto native = online_ ? runtimeInfo_.value("native_agent").toObject() : QJsonObject{};
    for (auto it = native.begin(); it != native.end(); ++it)
        adapters.append(it.key() + QStringLiteral(" · ") + it.value().toString());
    auto capability = tr("Native execution capability: %1")
                      .arg(adapters.isEmpty() ? tr("Unavailable") : adapters.join(QStringLiteral("\n")));
    const auto controls = online_ ? runtimeInfo_.value("execution_enforcement").toObject() : QJsonObject{};
    if (!controls.isEmpty()) {
        QStringList details;
        for (auto it = controls.begin(); it != controls.end(); ++it)
            details.append(it.key() + QStringLiteral(" · ") + it.value().toString());
        capability += QStringLiteral("\n\n") + tr("Execution controls: %1").arg(details.join(QStringLiteral("\n")));
    }
    runtimeCapabilities_->setText(capability);
    aboutDetails_->setText(tr("%1\nApplication %2\nRelease %3\n\n%4")
                              .arg(QStringLiteral(CGACP_PRODUCT_GENERATION), QStringLiteral(CGACP_APP_VERSION),
                                   QStringLiteral(CGACP_RELEASE_NAME), capability));
}

QJsonObject MainWindow::preferenceSettings() const
{
    const auto repoLayout = repositoryLayout();
    return {{"theme", themeChoice_->currentData().toString()}, {"density", densityChoice_->currentData().toString()},
            {"text_scale", textScale_->value()}, {"sidebar_width", sidebarWidth_},
            {"sidebar_collapsed", sidebarCollapsed_},
            {"repository_layout", repoLayout.isEmpty() ? QJsonValue(QJsonValue::Null) : QJsonValue(repoLayout)},
            {"notification_level", notificationChoice_->currentData().toString()}};
}

void MainWindow::applyPreferences(const QJsonObject &settings)
{
    applyingPreferences_ = true;
    const QSignalBlocker themeSignals(themeChoice_);
    const QSignalBlocker densitySignals(densityChoice_);
    const QSignalBlocker scaleSignals(textScale_);
    const QSignalBlocker notificationSignals(notificationChoice_);
    selectData(themeChoice_, settings.value("theme").toString("dark"));
    selectData(densityChoice_, settings.value("density").toString("comfortable"));
    textScale_->setValue(settings.value("text_scale").toInt(100));
    selectData(notificationChoice_, settings.value("notification_level").toString("all"));
    sidebarWidth_ = qBound(210, settings.value("sidebar_width").toInt(260), 360);
    sidebarCollapsed_ = settings.value("sidebar_collapsed").toBool();
    theme_->apply(themeChoice_->currentData().toString(), densityChoice_->currentData().toString(),
                  textScale_->value());
    changeSidebar(sidebarCollapsed_);
    restoreRepositoryLayout(settings.value("repository_layout").toObject());
    notifications_->setLevel(notificationChoice_->currentData().toString());
    applyingPreferences_ = false;
}

void MainWindow::loadPreferences()
{
    if (!online_ || preferencesLoading_ || preferencesSaving_)
        return;
    preferencesLoading_ = true;
    call("preferences.get", {}, [this](const QJsonObject &result, const QString &error) {
        preferencesLoading_ = false;
        if (!error.isEmpty() || !result.value("settings").isObject()) {
            preferencesRetryRequired_ = true;
            preferencesStatus_->setText(tr("Could not load saved preferences. Current settings are in memory."));
            preferencesStatus_->show();
            preferencesRetry_->show();
            showStatus(preferencesStatus_->text(), true);
            return;
        }
        const auto revision = result.value("revision").toInteger();
        const auto remote = result.value("settings").toObject();
        const auto local = preferenceSettings();
        const bool stale = preferencesLoaded_ && preferencesDirty_ &&
                           revision != preferencesRevision_;
        auto merged = remote;
        if (preferencesDirty_) {
            for (auto it = local.begin(); it != local.end(); ++it) {
                if (it.value() != savedPreferences_.value(it.key()))
                    merged.insert(it.key(), it.value());
            }
        }
        preferencesLoaded_ = true;
        preferencesRevision_ = revision;
        if (stale) {
            preferencesRetryRequired_ = true;
            preferencesStatus_->setText(tr("Preferences changed in another window. Review or retry your pending changes."));
            preferencesStatus_->show();
            preferencesRetry_->show();
            showStatus(preferencesStatus_->text(), true);
            return;
        }
        if (preferencesRetryRequired_ && preferencesDirty_)
            return;
        savedPreferences_ = remote;
        applyPreferences(merged);
        preferencesRetryRequired_ = false;
        preferencesRetry_->hide();
        preferencesStatus_->hide();
        if (preferencesDirty_)
            savePreferences();
    });
}

void MainWindow::schedulePreferencesSave()
{
    if (applyingPreferences_)
        return;
    preferencesDirty_ = true;
    if (!online_ || preferencesRetryRequired_) {
        preferencesStatus_->setText(tr("Preferences have unsaved changes."));
        preferencesStatus_->show();
        return;
    }
    preferencesTimer_->start();
}

void MainWindow::savePreferences()
{
    if (!preferencesDirty_ || preferencesSaving_ || preferencesLoading_ || !preferencesLoaded_ ||
        preferencesRetryRequired_ || !online_)
        return;
    const auto settings = preferenceSettings();
    preferencesSaving_ = true;
    call("preferences.set", {{"expected_revision", preferencesRevision_}, {"settings", settings}},
         [this, settings](const QJsonObject &result, const QString &error) {
        preferencesSaving_ = false;
        if (!error.isEmpty() || !result.value("settings").isObject()) {
            closeAfterSave_ = false;
            preferencesRetryRequired_ = true;
            preferencesStatus_->setText(tr("Could not save preferences. Your changes are still in this window."));
            preferencesStatus_->show();
            preferencesRetry_->show();
            showStatus(preferencesStatus_->text(), true);
            return;
        }
        preferencesRevision_ = result.value("revision").toInteger();
        savedPreferences_ = result.value("settings").toObject();
        preferencesDirty_ = preferenceSettings() != settings;
        preferencesStatus_->hide();
        preferencesRetry_->hide();
        if (preferencesDirty_) {
            preferencesTimer_->start();
        } else if (closeAfterSave_) {
            closeAfterSave_ = false;
            stopRuntimeAndClose();
        }
    });
}
