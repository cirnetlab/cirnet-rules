#include "main_window.h"
#include "build_info.h"
#include "notification_center.h"

#include <QApplication>
#include <QButtonGroup>
#include <QCheckBox>
#include <QClipboard>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPainter>
#include <QPainterPath>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QRadioButton>
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

QString themeMode(const QButtonGroup &choices)
{
    const auto *checked = choices.checkedButton();
    return checked ? checked->property("themeMode").toString() : QStringLiteral("dark");
}

void selectTheme(const QButtonGroup &choices, const QString &key)
{
    for (auto *button : choices.buttons()) {
        if (button->property("themeMode").toString() == key) {
            button->setChecked(true);
            return;
        }
    }
}

class ThemeTile final : public QRadioButton {
public:
    ThemeTile(const QString &title, const QString &key, const QString &name)
        : QRadioButton(title), mode_(key)
    {
        setObjectName(name);
        setProperty("themeMode", key);
        setAccessibleName(title);
        setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
        setMaximumWidth(145);
        setAttribute(Qt::WA_Hover);
    }

    QSize sizeHint() const override { return {124, 60 + 2 * fontMetrics().lineSpacing()}; }
    QSize minimumSizeHint() const override { return {70, sizeHint().height()}; }

protected:
    bool hitButton(const QPoint &position) const override { return rect().contains(position); }

    void paintEvent(QPaintEvent *) override
    {
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing);
        const auto colors = palette();
        auto border = colors.color(QPalette::WindowText);
        border.setAlpha(55);
        const auto surface = colors.color(underMouse() || isDown() ? QPalette::Button : QPalette::Base);
        painter.setBrush(surface);
        painter.setPen(QPen(isChecked() ? colors.color(QPalette::Highlight) : border, isChecked() ? 2 : 1));
        painter.drawRoundedRect(QRectF(rect()).adjusted(1, 1, -1, -1), 9, 9);

        const QRectF preview(8, 8, width() - 16, 42);
        QPainterPath clip;
        clip.addRoundedRect(preview, 5, 5);
        painter.save();
        painter.setClipPath(clip);
        if (mode_ == "system") {
            painter.save();
            painter.setClipRect(QRectF(preview.topLeft(), QSizeF(preview.width() / 2, preview.height())),
                                Qt::IntersectClip);
            drawPreview(painter, preview, true);
            painter.restore();
            painter.setClipRect(QRectF(QPointF(preview.center().x(), preview.top()),
                                      QSizeF(preview.width() / 2, preview.height())), Qt::IntersectClip);
            drawPreview(painter, preview, false);
        } else {
            drawPreview(painter, preview, mode_ == "light");
        }
        painter.restore();

        painter.setPen(colors.color(isEnabled() ? QPalette::Active : QPalette::Disabled, QPalette::WindowText));
        painter.drawText(QRect(6, 54, width() - 12, height() - 58),
                         Qt::AlignHCenter | Qt::AlignTop | Qt::TextWordWrap, text());
        if (property("keyboardFocusVisible").toBool()) {
            painter.setPen(QPen(colors.color(QPalette::WindowText), 1, Qt::DotLine));
            painter.setBrush(Qt::NoBrush);
            painter.drawRoundedRect(QRectF(rect()).adjusted(4, 4, -4, -4), 6, 6);
        }
    }

private:
    static void drawPreview(QPainter &painter, const QRectF &area, bool light)
    {
        const auto rail = light ? QColor("#dce3e0") : QColor("#303a37");
        const auto text = light ? QColor("#7a8883") : QColor("#9daaa6");
        const auto accent = light ? QColor("#007b69") : QColor("#70d7c6");
        const auto panelWidth = area.width() * 0.31;
        const auto x = area.left() + panelWidth + 6;
        const auto lineWidth = area.right() - x - 6;
        painter.fillRect(area, light ? QColor("#edf1ef") : QColor("#1c2522"));
        painter.fillRect(QRectF(area.topLeft(), QSizeF(panelWidth, area.height())), rail);
        painter.fillRect(QRectF(area.left() + 4, area.top() + 7, panelWidth - 8, 3), text);
        painter.fillRect(QRectF(area.left() + 4, area.top() + 15, panelWidth - 8, 5), accent);
        painter.fillRect(QRectF(x, area.top() + 8, lineWidth * .68, 3), text);
        painter.fillRect(QRectF(x, area.top() + 17, lineWidth, 2), text);
        painter.fillRect(QRectF(x, area.top() + 24, lineWidth * .78, 2), text);
        painter.fillRect(QRectF(x, area.bottom() - 10, lineWidth * .43, 4), accent);
    }

    QString mode_;
};

QFrame *settingsGroup()
{
    auto *group = new QFrame;
    group->setObjectName("settingsGroup");
    return group;
}

void addSettingRow(QGridLayout &rows, int row, const QString &name, const QString &description, QWidget *control)
{
    if (row > 0) {
        auto *line = new QFrame;
        line->setObjectName("settingsSeparator");
        line->setFixedHeight(1);
        rows.addWidget(line, row * 2 - 1, 0, 1, 2);
    }
    auto *copy = new QVBoxLayout;
    copy->setSpacing(2);
    auto *heading = label(name, "sectionTitle");
    heading->setBuddy(control);
    copy->addWidget(heading);
    copy->addWidget(label(description, "meta"));
    rows.addLayout(copy, row * 2, 0);
    rows.addWidget(control, row * 2, 1, Qt::AlignRight | Qt::AlignVCenter);
}
}

QWidget *MainWindow::makeSettings()
{
    auto *page = new QWidget;
    page->setObjectName("settingsPage");
    auto *shell = new QHBoxLayout(page);
    shell->setContentsMargins(0, 0, 0, 0);
    shell->setSpacing(0);

    auto *sidebar = new QFrame;
    sidebar->setObjectName("settingsSidebar");
    sidebar->setMinimumWidth(155);
    sidebar->setMaximumWidth(200);
    auto *categories = new QVBoxLayout(sidebar);
    categories->setContentsMargins(12, 18, 12, 12);
    categories->setSpacing(12);
    categories->addWidget(label(tr("Settings"), "heading"));
    settingsSearch_ = new QLineEdit;
    settingsSearch_->setObjectName("settingsSearch");
    settingsSearch_->setProperty("searchField", true);
    settingsSearch_->setPlaceholderText(tr("Search settings"));
    settingsSearch_->setAccessibleName(tr("Search settings"));
    settingsSearch_->setClearButtonEnabled(true);
    categories->addWidget(settingsSearch_);
    settingsNavigation_ = new QListWidget;
    settingsNavigation_->setObjectName("settingsNavigation");
    settingsNavigation_->setAccessibleName(tr("Settings section"));
    settingsNavigation_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    settingsNavigation_->setTextElideMode(Qt::ElideRight);
    categories->addWidget(settingsNavigation_, 1);
    auto *noResults = label(tr("No matching settings."), "meta");
    noResults->hide();
    categories->addWidget(noResults);
    auto *back = new QPushButton(tr("Back"));
    back->setObjectName("settingsBack");
    back->setAccessibleName(tr("Back to workspace"));
    back->setToolTip(tr("Back to workspace"));
    connect(back, &QPushButton::clicked, this, &MainWindow::closeSettings);
    categories->addWidget(back);
    shell->addWidget(sidebar);

    auto *content = new QWidget;
    auto *layout = new QVBoxLayout(content);
    layout->setContentsMargins(16, 20, 16, 12);
    layout->setSpacing(10);
    settingsPages_ = new QStackedWidget;
    settingsPages_->setObjectName("settingsPages");
    settingsPages_->setMinimumWidth(0);
    settingsPages_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Expanding);
    layout->addWidget(settingsPages_, 1);
    shell->addWidget(content, 1);
    const auto section = [this](const QString &title, const QString &key, const QString &keywords, QWidget *panel) {
        auto *item = new QListWidgetItem(title, settingsNavigation_);
        item->setData(Qt::UserRole, key);
        item->setData(Qt::UserRole + 1, title + ' ' + keywords);
        item->setToolTip(title);
        settingsPages_->addWidget(panel);
    };

    auto *appearance = new QWidget;
    appearance->setObjectName("appearancePage");
    auto *appearanceCenter = new QHBoxLayout(appearance);
    appearanceCenter->setContentsMargins(0, 0, 0, 0);
    auto *appearancePanel = new QWidget;
    appearancePanel->setMaximumWidth(850);
    appearanceCenter->addStretch();
    appearanceCenter->addWidget(appearancePanel, 100, Qt::AlignTop);
    appearanceCenter->addStretch();
    auto *appearanceLayout = new QVBoxLayout(appearancePanel);
    appearanceLayout->setContentsMargins(2, 2, 2, 12);
    appearanceLayout->setSpacing(12);
    appearanceLayout->addWidget(label(tr("Appearance"), "heading"));
    appearanceLayout->addWidget(label(tr("Customize the desktop layout, theme and text size."), "lead"));
    appearanceLayout->addSpacing(6);
    appearanceLayout->addWidget(label(tr("Visual style"), "sectionTitle"));
    auto *themePanel = settingsGroup();
    auto *previews = new QHBoxLayout(themePanel);
    previews->setContentsMargins(12, 10, 12, 10);
    previews->setSpacing(8);
    previews->addStretch();
    themeChoice_ = new QButtonGroup(appearance);
    const auto addMode = [this, previews](const QString &title, const QString &key, const QString &name) {
        auto *tile = new ThemeTile(title, key, name);
        themeChoice_->addButton(tile);
        previews->addWidget(tile, 1);
    };
    addMode(tr("System"), "system", "themeSystem");
    addMode(tr("Light"), "light", "themeLight");
    addMode(tr("Dark"), "dark", "themeDark");
    previews->addStretch();
    selectTheme(*themeChoice_, "dark");
    appearanceLayout->addWidget(themePanel);

    appearanceLayout->addSpacing(6);
    appearanceLayout->addWidget(label(tr("Layout and typography"), "sectionTitle"));
    auto *layoutPanel = settingsGroup();
    auto *rows = new QGridLayout(layoutPanel);
    rows->setContentsMargins(12, 12, 12, 12);
    rows->setHorizontalSpacing(12);
    rows->setVerticalSpacing(10);
    rows->setColumnStretch(0, 1);
    densityChoice_ = new QComboBox;
    densityChoice_->setObjectName("densityChoice");
    densityChoice_->addItem(tr("Comfortable"), "comfortable");
    densityChoice_->addItem(tr("Compact"), "compact");
    densityChoice_->setSizeAdjustPolicy(QComboBox::AdjustToContents);
    textScale_ = new QSpinBox;
    textScale_->setObjectName("textScale");
    textScale_->setRange(85, 150);
    textScale_->setSingleStep(5);
    textScale_->setValue(100);
    textScale_->setSuffix(QStringLiteral("%"));
    reducedMotion_ = new QCheckBox;
    reducedMotion_->setObjectName("reducedMotion");
    reducedMotion_->setAccessibleName(tr("Reduced motion"));
    addSettingRow(*rows, 0, tr("&Density"), tr("Adjust the spacing between controls."), densityChoice_);
    addSettingRow(*rows, 1, tr("Text &size"), tr("Scale text throughout the desktop."), textScale_);
    addSettingRow(*rows, 2, tr("&Reduced motion"), tr("Skip motion effects while keeping visual feedback."), reducedMotion_);
    appearanceLayout->addWidget(layoutPanel);
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
    section(tr("Appearance"), "appearance", tr("theme density text size reduced motion reset layout system light dark"),
            scrollPanel(appearance));
    section(tr("Repositories"), "repositories", tr("binding folder path worktree relink archive"), repositoriesPage_);

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
    section(tr("Agents & Runtime"), "runtime", tr("connection reconnect diagnostics capability"), scrollPanel(runtime));

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
    section(tr("Notifications"), "notifications", tr("toast history warnings errors"), notices);

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
    section(tr("About"), "about", tr("version release application identity"), scrollPanel(about));

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
    connect(settingsNavigation_, &QListWidget::currentRowChanged, this, [this](int index) {
        if (index < 0)
            return;
        ++navigationRevision_;
        settingsPages_->setCurrentIndex(index);
        if (settingsSection() == "notifications")
            notifications_->markHistoryRead();
        else if (settingsSection() == "repositories")
            QTimer::singleShot(0, this, &MainWindow::restoreRepositoryScroll);
    });
    settingsNavigation_->setCurrentRow(0);
    connect(settingsSearch_, &QLineEdit::textChanged, this, [this, noResults](const QString &text) {
        bool matched = false;
        for (int row = 0; row < settingsNavigation_->count(); ++row) {
            auto *item = settingsNavigation_->item(row);
            const bool visible = item->data(Qt::UserRole + 1).toString().contains(text.trimmed(), Qt::CaseInsensitive);
            item->setHidden(!visible);
            matched |= visible;
        }
        noResults->setVisible(!matched);
    });
    connect(settingsSearch_, &QLineEdit::returnPressed, this, [this] {
        auto *match = settingsNavigation_->currentItem();
        if (!match || match->isHidden()) {
            match = nullptr;
            for (int row = 0; row < settingsNavigation_->count(); ++row) {
                if (auto *item = settingsNavigation_->item(row); !item->isHidden()) {
                    match = item;
                    break;
                }
            }
        }
        if (!match)
            return;
        settingsNavigation_->setCurrentItem(match);
        const auto query = settingsSearch_->text().trimmed();
        QWidget *target = settingsNavigation_;
        if (!query.isEmpty() && settingsSection() == "appearance") {
            if (tr("Reduced motion").contains(query, Qt::CaseInsensitive))
                target = reducedMotion_;
            else if (tr("Text size").contains(query, Qt::CaseInsensitive))
                target = textScale_;
            else if (tr("Density").contains(query, Qt::CaseInsensitive))
                target = densityChoice_;
            else if (tr("Theme mode").contains(query, Qt::CaseInsensitive)) {
                if (auto *selected = themeChoice_->checkedButton())
                    target = selected;
            }
        }
        target->setFocus(Qt::ShortcutFocusReason);
    });
    const auto visualChange = [this] {
        theme_->apply(themeMode(*themeChoice_), densityChoice_->currentData().toString(),
                      textScale_->value());
        notifications_->setLevel(notificationChoice_->currentData().toString());
        notifications_->setReducedMotion(reducedMotion_->isChecked());
        schedulePreferencesSave();
    };
    connect(themeChoice_, &QButtonGroup::buttonToggled, this, [visualChange](QAbstractButton *, bool checked) {
        if (checked)
            visualChange();
    });
    connect(densityChoice_, &QComboBox::currentIndexChanged, this, visualChange);
    connect(textScale_, &QSpinBox::valueChanged, this, visualChange);
    connect(notificationChoice_, &QComboBox::currentIndexChanged, this, visualChange);
    connect(reducedMotion_, &QCheckBox::toggled, this, visualChange);
    return page;
}

QString MainWindow::settingsSection() const
{
    const auto *current = settingsNavigation_->currentItem();
    return current ? current->data(Qt::UserRole).toString() : QString{};
}

void MainWindow::selectSettingsSection(const QString &section)
{
    for (int row = 0; row < settingsNavigation_->count(); ++row) {
        auto *item = settingsNavigation_->item(row);
        if (item->data(Qt::UserRole).toString() != section)
            continue;
        if (item->isHidden())
            settingsSearch_->clear();
        settingsNavigation_->setCurrentItem(item);
        settingsNavigation_->scrollToItem(item);
        return;
    }
}

void MainWindow::showSettings(const QString &section)
{
    const bool entering = pages_->currentWidget() != settingsPage_;
    if (entering) {
        previousPage_ = pages_->currentWidget();
        settingsReturnFocus_ = QApplication::focusWidget();
    }
    if (!section.isEmpty())
        selectSettingsSection(section);
    showPage(settingsPage_);
    if (entering)
        settingsNavigation_->setFocus(Qt::ShortcutFocusReason);
    if (settingsSection() == "notifications")
        notifications_->markHistoryRead();
    else if (settingsSection() == "repositories")
        QTimer::singleShot(0, this, &MainWindow::restoreRepositoryScroll);
}

void MainWindow::closeSettings()
{
    if (pages_->currentWidget() != settingsPage_ || QApplication::activePopupWidget() ||
        QApplication::activeModalWidget())
        return;
    const auto focus = settingsReturnFocus_;
    settingsReturnFocus_.clear();
    showPage(previousPage_ ? previousPage_ : automationsPage_);
    if (focus && focus->isVisible() && focus->isEnabled() && focus->window() == this &&
        !QApplication::activeModalWidget())
        focus->setFocus(Qt::OtherFocusReason);
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
    return {{"theme", themeMode(*themeChoice_)}, {"density", densityChoice_->currentData().toString()},
            {"text_scale", textScale_->value()}, {"sidebar_width", sidebarWidth_},
            {"sidebar_collapsed", sidebarCollapsed_},
            {"reduced_motion", reducedMotion_->isChecked()},
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
    const QSignalBlocker motionSignals(reducedMotion_);
    selectTheme(*themeChoice_, settings.value("theme").toString("dark"));
    selectData(densityChoice_, settings.value("density").toString("comfortable"));
    textScale_->setValue(settings.value("text_scale").toInt(100));
    reducedMotion_->setChecked(settings.value("reduced_motion").toBool());
    selectData(notificationChoice_, settings.value("notification_level").toString("all"));
    sidebarWidth_ = qBound(210, settings.value("sidebar_width").toInt(260), 360);
    sidebarCollapsed_ = settings.value("sidebar_collapsed").toBool();
    theme_->apply(themeMode(*themeChoice_), densityChoice_->currentData().toString(),
                  textScale_->value());
    changeSidebar(sidebarCollapsed_);
    restoreRepositoryLayout(settings.value("repository_layout").toObject());
    notifications_->setLevel(notificationChoice_->currentData().toString());
    notifications_->setReducedMotion(reducedMotion_->isChecked());
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
