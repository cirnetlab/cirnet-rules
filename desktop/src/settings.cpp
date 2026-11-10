#include "main_window.h"
#include "governance_panel.h"
#include "live_text.h"
#include "build_info.h"
#include "notification_center.h"
#include "settings_panels.h"

#include <QApplication>
#include <QButtonGroup>
#include <QCheckBox>
#include <QClipboard>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFontDatabase>
#include <QFormLayout>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QKeyEvent>
#include <QLineEdit>
#include <QListWidget>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QRadioButton>
#include <QRegularExpressionValidator>
#include <QScreen>
#include <QScrollArea>
#include <QSignalBlocker>
#include <QSpinBox>
#include <QStackedWidget>
#include <QStyledItemDelegate>
#include <QTimer>
#include <QToolButton>
#include <QToolTip>
#include <QVBoxLayout>

namespace {
constexpr int baseTextPixels = 13;
constexpr int baseDensityPixels = 6;
constexpr int installedFontRole = Qt::UserRole + 1;
const QJsonObject defaultAppearance{
    {"theme", "dark"}, {"density", "comfortable"}, {"text_scale", 100}, {"reduced_motion", false},
    {"appearance", QJsonObject{{"accent", "cirnet"}, {"ui_font", "system"},
                               {"interface_px", baseTextPixels}, {"conversation_px", baseTextPixels},
                               {"code_px", baseTextPixels}, {"density_px", baseDensityPixels},
                               {"content_scale", 0}, {"code_scale", 0}, {"language", "en"}}},
};

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

int textPixels(int legacyPercent)
{
    return qBound(10, qRound(baseTextPixels * legacyPercent / 100.0), 26);
}

int compatibleScale(int pixels)
{
    return qBound(85, qRound(pixels * 100.0 / baseTextPixels), 150);
}

QJsonObject normalizedAppearance(const QJsonObject &settings)
{
    auto style = settings.value("appearance").toObject();
    const int interfacePx = style.value("interface_px").toInt(textPixels(settings.value("text_scale").toInt(100)));
    const auto size = [interfacePx, &style](const char *pixels, const char *percent) {
        const int legacy = style.value(QLatin1StringView(percent)).toInt();
        return style.value(QLatin1StringView(pixels)).toInt(legacy == 0 ? interfacePx : textPixels(legacy));
    };
    const int conversationPx = size("conversation_px", "content_scale");
    const int codePx = size("code_px", "code_scale");
    const int densityPx = style.value("density_px").toInt(settings.value("density") == "compact" ? 4 : baseDensityPixels);
    style.insert("interface_px", interfacePx);
    style.insert("conversation_px", conversationPx);
    style.insert("code_px", codePx);
    style.insert("density_px", densityPx);
    style.insert("accent", style.value("accent").toString("cirnet"));
    style.insert("ui_font", style.value("ui_font").toString("system"));
    style.insert("language", style.value("language").toString("en"));
    style.insert("content_scale", conversationPx == interfacePx ? 0 : compatibleScale(conversationPx));
    style.insert("code_scale", codePx == interfacePx ? 0 : compatibleScale(codePx));
    return {{"theme", settings.value("theme").toString("dark")},
            {"density", densityPx <= 4 ? "compact" : "comfortable"},
            {"text_scale", compatibleScale(interfacePx)},
            {"reduced_motion", settings.value("reduced_motion").toBool()},
            {"appearance", style}};
}

void setPreferenceRevert(QWidget *page, bool visible)
{
    page->findChild<QPushButton *>("preferencesRevert")->setVisible(visible);
}

QString themeMode(const QButtonGroup &choices)
{
    const auto *checked = choices.checkedButton();
    return checked ? checked->property("themeMode").toString() : defaultAppearance.value("theme").toString();
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
        setMaximumWidth(112);
        setAttribute(Qt::WA_Hover);
    }

    QSize sizeHint() const override { return {108, 59 + 2 * fontMetrics().lineSpacing()}; }
    QSize minimumSizeHint() const override { return {64, sizeHint().height()}; }

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
        const bool keyboardFocus = property("keyboardFocusVisible").toBool();
        const int outline = keyboardFocus ? 3 : isChecked() ? 2 : 1;
        painter.setBrush(surface);
        painter.setPen(QPen(keyboardFocus ? colors.color(QPalette::Link)
                                          : isChecked() ? colors.color(QPalette::Highlight) : border, outline));
        const auto inset = outline / 2.0;
        painter.drawRoundedRect(QRectF(rect()).adjusted(inset, inset, -inset, -inset), 12, 12);

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
    }

private:
    static void drawPreview(QPainter &painter, const QRectF &area, bool light)
    {
        const auto rail = light ? QColor("#dce3e0") : QColor("#181818");
        const auto text = light ? QColor("#7a8883") : QColor("#a3a3a3");
        const auto accent = appearanceAccent("cirnet", light);
        const auto panelWidth = area.width() * 0.31;
        const auto x = area.left() + panelWidth + 6;
        const auto lineWidth = area.right() - x - 6;
        painter.fillRect(area, defaultAppearanceBackground(light));
        painter.fillRect(QRectF(area.topLeft(), QSizeF(panelWidth, area.height())), rail);
        painter.fillRect(QRectF(area.left() + 4, area.top() + 7, panelWidth - 8, 3), text);
        painter.fillRect(QRectF(area.left() + 4, area.top() + 15, panelWidth - 8, 5),
                         light ? accent : QColor("#2a2a2a"));
        painter.fillRect(QRectF(x, area.top() + 8, lineWidth * .68, 3), text);
        painter.fillRect(QRectF(x, area.top() + 17, lineWidth, 2), text);
        painter.fillRect(QRectF(x, area.top() + 24, lineWidth * .78, 2), text);
        painter.fillRect(QRectF(x, area.bottom() - 10, lineWidth * .43, 4), accent);
    }

    QString mode_;
};

QIcon colorDot(const QColor &color)
{
    QPixmap image(32, 32);
    image.setDevicePixelRatio(2);
    image.fill(Qt::transparent);
    QPainter painter(&image);
    painter.setRenderHint(QPainter::Antialiasing);
    painter.setPen(QApplication::palette().color(QPalette::Mid));
    painter.setBrush(color);
    painter.drawEllipse(QRectF(2, 2, 12, 12));
    return QIcon(image);
}

class AppearanceChoiceDelegate final : public QStyledItemDelegate {
public:
    explicit AppearanceChoiceDelegate(QComboBox *choice) : QStyledItemDelegate(choice), choice_(choice) {}

    QSize sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index) const override
    {
        return QStyledItemDelegate::sizeHint(option, index) + QSize(20, 0);
    }

    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override
    {
        QStyledItemDelegate::paint(painter, option, index);
        if (index.row() != choice_->currentIndex())
            return;
        painter->save();
        painter->setRenderHint(QPainter::Antialiasing);
        painter->setPen(QPen(option.palette.color(QPalette::Text), 1.5, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        const QPointF anchor(option.rect.right() - 12, option.rect.center().y());
        painter->drawPolyline(QPolygonF{anchor + QPointF(-4, 0), anchor + QPointF(-1, 3), anchor + QPointF(5, -4)});
        painter->restore();
    }

private:
    QComboBox *choice_;
};

struct PickerColor {
    int hue;
    int saturation;
    int value;
};

class ColorRamp final : public QWidget {
public:
    explicit ColorRamp(bool hueOnly) : hueOnly_(hueOnly)
    {
        setFocusPolicy(Qt::StrongFocus);
        setFixedHeight(hueOnly ? 20 : 138);
        setCursor(Qt::CrossCursor);
    }

    void setColor(PickerColor color)
    {
        color_ = color;
        update();
    }
    void setChange(std::function<void(PickerColor)> change) { change_ = std::move(change); }

protected:
    void paintEvent(QPaintEvent *) override
    {
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing);
        const QRectF box = QRectF(rect()).adjusted(1, 1, -1, -1);
        QPainterPath clip;
        clip.addRoundedRect(box, 7, 7);
        painter.setClipPath(clip);
        QLinearGradient horizontal(box.topLeft(), box.topRight());
        QPointF marker;
        if (hueOnly_) {
            for (int step = 0; step <= 6; ++step)
                horizontal.setColorAt(step / 6.0, QColor::fromHsv((step * 60) % 360, 255, 255));
            painter.fillRect(box, horizontal);
            marker = {box.left() + box.width() * color_.hue / 359, box.center().y()};
        } else {
            horizontal.setColorAt(0, Qt::white);
            horizontal.setColorAt(1, QColor::fromHsv(color_.hue, 255, 255));
            painter.fillRect(box, horizontal);
            QLinearGradient vertical(box.topLeft(), box.bottomLeft());
            vertical.setColorAt(0, Qt::transparent);
            vertical.setColorAt(1, Qt::black);
            painter.fillRect(box, vertical);
            marker = {box.left() + box.width() * color_.saturation / 255,
                      box.top() + box.height() * (255 - color_.value) / 255};
        }
        const auto markerBounds = box.adjusted(6, 6, -6, -6);
        marker.setX(qBound(markerBounds.left(), marker.x(), markerBounds.right()));
        marker.setY(qBound(markerBounds.top(), marker.y(), markerBounds.bottom()));
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::black, 3));
        painter.drawEllipse(marker, 5, 5);
        painter.setPen(QPen(Qt::white, 1.5));
        painter.drawEllipse(marker, 5, 5);
        if (hasFocus()) {
            painter.setClipping(false);
            painter.setPen(QPen(palette().color(QPalette::Link), 1));
            painter.drawPath(clip);
        }
    }

    void mousePressEvent(QMouseEvent *event) override
    {
        if (event->button() == Qt::LeftButton) {
            setFocus(Qt::MouseFocusReason);
            moveTo(event->position());
        }
    }

    void mouseMoveEvent(QMouseEvent *event) override
    {
        if (event->buttons() & Qt::LeftButton)
            moveTo(event->position());
    }

    void keyPressEvent(QKeyEvent *event) override
    {
        int horizontal = 0;
        int vertical = 0;
        switch (event->key()) {
        case Qt::Key_Left: horizontal = -1; break;
        case Qt::Key_Right: horizontal = 1; break;
        case Qt::Key_Up: vertical = 1; break;
        case Qt::Key_Down: vertical = -1; break;
        default:
            QWidget::keyPressEvent(event);
            return;
        }
        auto next = color_;
        if (hueOnly_)
            next.hue = (color_.hue + (horizontal + vertical) * 3 + 360) % 360;
        else {
            next.saturation = qBound(0, color_.saturation + horizontal * 5, 255);
            next.value = qBound(0, color_.value + vertical * 5, 255);
        }
        change_(next);
    }

    void focusInEvent(QFocusEvent *event) override { QWidget::focusInEvent(event); update(); }
    void focusOutEvent(QFocusEvent *event) override { QWidget::focusOutEvent(event); update(); }

private:
    void moveTo(const QPointF &position)
    {
        const auto x = qBound(0.0, (position.x() - 1) / (width() - 2), 1.0);
        const auto y = qBound(0.0, (position.y() - 1) / (height() - 2), 1.0);
        auto next = color_;
        if (hueOnly_)
            next.hue = qRound(359 * x);
        else {
            next.saturation = qRound(255 * x);
            next.value = qRound(255 * (1 - y));
        }
        change_(next);
    }

    bool hueOnly_;
    std::function<void(PickerColor)> change_;
    PickerColor color_{0, 0, 0};
};
}

class AppearanceColorField final : public QFrame {
public:
    AppearanceColorField(const QString &valueName, live::Phrase title, QPalette::ColorRole role,
                         bool clearable, std::function<void()> change)
        : title_(std::move(title)), role_(role), clearable_(clearable), change_(std::move(change))
    {
        setObjectName("appearanceSwatch");
        auto *row = new QHBoxLayout(this);
        row->setContentsMargins(5, 3, 8, 3);
        row->setSpacing(3);
        dot_ = new QToolButton;
        dot_->setObjectName(valueName + "Picker");
        dot_->setFixedSize(26, 26);
        dot_->setIconSize({16, 16});
        dot_->setStyleSheet(QStringLiteral("QToolButton { padding: 0; }"));
        row->addWidget(dot_);
        value_ = new QLineEdit;
        value_->setObjectName(valueName);
        live::bind(this, [this] {
            const auto title = title_();
            dot_->setAccessibleName(MainWindow::tr("Choose %1").arg(title));
            dot_->setToolTip(dot_->accessibleName());
            value_->setAccessibleName(title);
            value_->setToolTip(clearable_
                ? MainWindow::tr("Enter a six-digit hexadecimal color. Clear the field to use the theme color.")
                : MainWindow::tr("Enter a six-digit hexadecimal color."));
        });
        value_->setMaxLength(7);
        value_->setValidator(new QRegularExpressionValidator(QRegularExpression(QStringLiteral("^#?[A-Fa-f0-9]{0,6}$")), value_));
        value_->setStyleSheet(QStringLiteral(
            "QLineEdit { border: 1px solid transparent; background: transparent; border-radius: 5px; padding: 2px 4px; }"
            "QLineEdit:focus { border-color: palette(link); }"));
        row->addWidget(value_);
        setFocusProxy(value_);
        setFocusPolicy(Qt::StrongFocus);
        connect(dot_, &QToolButton::clicked, this, [this] { openPicker(); });
        connect(value_, &QLineEdit::textChanged, this, [this] { refreshSwatch(); });
        connect(value_, &QLineEdit::editingFinished, this, [this] { commitText(); });
        refresh();
    }

    QString stored() const { return stored_; }
    QLineEdit *editor() const { return value_; }

    void load(const QString &color)
    {
        stored_ = color.toUpper();
        refresh(true);
    }

    void refresh(bool force = false)
    {
        const auto color = effectiveColor();
        if (force || (!value_->isModified() && value_->text() != color.name().toUpper()))
            value_->setText(color.name().toUpper());
        refreshSwatch();
        const QFontMetrics metrics(QApplication::font());
        int digitWidth = 0;
        for (const auto digit : QStringLiteral("0123456789ABCDEF"))
            digitWidth = qMax(digitWidth, metrics.horizontalAdvance(digit));
        value_->setFixedWidth(metrics.horizontalAdvance(u'#') + 6 * digitWidth + 24);
    }

private:
    QColor effectiveColor() const
    {
        return stored_.isEmpty() ? QApplication::palette().color(role_) : QColor(stored_);
    }

    QColor enteredColor() const
    {
        auto hex = value_->text();
        if (!hex.startsWith(u'#'))
            hex.prepend(u'#');
        return hex.size() == 7 ? QColor(hex) : QColor{};
    }

    void refreshSwatch()
    {
        const auto entered = enteredColor();
        dot_->setIcon(colorDot(entered.isValid() ? entered : effectiveColor()));
    }

    void commitText()
    {
        const auto hex = value_->text().toUpper();
        if (!value_->isModified() && hex == effectiveColor().name().toUpper()) {
            refresh(true);
            return;
        }
        if (hex.isEmpty() && clearable_) {
            setStored({});
            return;
        }
        const auto entered = enteredColor();
        if (entered.isValid()) {
            setStored(entered.name().toUpper());
        } else {
            refresh(true);
            QToolTip::showText(value_->mapToGlobal(QPoint(0, value_->height())), value_->toolTip(), value_);
        }
    }

    void setStored(const QString &color)
    {
        const bool changed = stored_ != color;
        load(color);
        if (changed)
            change_();
    }

    void openPicker()
    {
        if (auto *existing = dot_->findChild<QDialog *>("appearanceColorPicker")) {
            if (existing->isVisible()) {
                existing->raise();
                return;
            }
            existing->deleteLater();
        }
        auto *picker = new QDialog(dot_, Qt::Popup);
        picker->setObjectName("appearanceColorPicker");
        picker->setWindowTitle(dot_->accessibleName());
        picker->setAccessibleName(picker->windowTitle());
        picker->setAttribute(Qt::WA_DeleteOnClose);
        picker->setMinimumWidth(272);
        auto *layout = new QVBoxLayout(picker);
        layout->setContentsMargins(12, 10, 12, 10);
        layout->setSpacing(8);
        layout->addWidget(label(title_(), "sectionTitle"));
        const auto seed = effectiveColor();
        auto *preview = label(seed.name().toUpper(), "meta");
        auto update = [preview](ColorRamp *plane, ColorRamp *hue, PickerColor color) {
            plane->setColor(color);
            hue->setColor(color);
            preview->setText(QColor::fromHsv(color.hue, color.saturation, color.value).name().toUpper());
        };
        auto *plane = new ColorRamp(false);
        auto *hue = new ColorRamp(true);
        plane->setObjectName("appearanceColorPlane");
        plane->setAccessibleName(MainWindow::tr("Saturation and brightness"));
        hue->setObjectName("appearanceColorHue");
        hue->setAccessibleName(MainWindow::tr("Hue"));
        layout->addWidget(plane);
        layout->addWidget(hue);
        auto *actions = new QHBoxLayout;
        actions->addWidget(preview, 1);
        auto *cancel = new QPushButton(MainWindow::tr("Cancel"));
        auto *apply = new QPushButton(MainWindow::tr("Apply"));
        apply->setObjectName("applyAppearanceColor");
        apply->setProperty("primary", true);
        actions->addWidget(cancel);
        actions->addWidget(apply);
        layout->addLayout(actions);
        connect(cancel, &QPushButton::clicked, picker, &QDialog::reject);
        connect(picker, &QDialog::finished, dot_, [control = dot_, popup = QPointer(picker)] {
            QTimer::singleShot(0, control, [control, popup] {
                auto *owner = control->window();
                const auto *active = QApplication::activeWindow();
                if (QApplication::applicationState() != Qt::ApplicationActive ||
                    (active && active != owner && active != popup.data()))
                    return;
                if (active != owner)
                    owner->activateWindow();
                if (!QApplication::focusWidget() || QApplication::focusWidget()->window() == popup.data())
                    control->setFocus(Qt::PopupFocusReason);
            });
        });
        connect(apply, &QPushButton::clicked, picker, [this, picker, preview] {
            const auto value = preview->text();
            picker->accept();
            setStored(value);
        });
        plane->setChange([update, plane, hue](PickerColor color) { update(plane, hue, color); });
        hue->setChange([update, plane, hue](PickerColor color) { update(plane, hue, color); });
        update(plane, hue, {qMax(0, seed.hsvHue()), seed.hsvSaturation(), seed.value()});
        preview->setText(seed.name().toUpper());
        picker->adjustSize();
        const auto anchor = dot_->mapToGlobal(QPoint(dot_->width(), dot_->height() + 4));
        const auto available = dot_->screen()->availableGeometry();
        const int x = qBound(available.left(), anchor.x() - picker->width(),
                             qMax(available.left(), available.right() - picker->width() + 1));
        const int preferredY = anchor.y() + picker->height() <= available.bottom() + 1
            ? anchor.y() : dot_->mapToGlobal(QPoint(0, 0)).y() - picker->height() - 4;
        const int y = qBound(available.top(), preferredY,
                             qMax(available.top(), available.bottom() - picker->height() + 1));
        picker->move(x, y);
        picker->show();
        plane->setFocus();
    }

    live::Phrase title_;
    QPalette::ColorRole role_;
    bool clearable_;
    std::function<void()> change_;
    QString stored_;
    QToolButton *dot_;
    QLineEdit *value_;
};

namespace {
QFrame *settingsGroup()
{
    auto *group = new QFrame;
    group->setObjectName("settingsGroup");
    return group;
}

void addSettingRow(QGridLayout &rows, int row, live::Phrase name, live::Phrase description, QWidget *control)
{
    if (row > 0) {
        auto *line = new QFrame;
        line->setObjectName("settingsSeparator");
        line->setFixedHeight(1);
        rows.addWidget(line, row * 2 - 1, 0, 1, 2);
    }
    auto *copy = new QVBoxLayout;
    copy->setSpacing(2);
    auto *heading = live::text(label({}, "sectionTitle"), std::move(name));
    if (control->focusPolicy() != Qt::NoFocus)
        heading->setBuddy(control);
    copy->addWidget(heading);
    if (description)
        copy->addWidget(live::text(label({}, "meta"), std::move(description)));
    rows.addLayout(copy, row * 2, 0);
    rows.addWidget(control, row * 2, 1, Qt::AlignRight | Qt::AlignVCenter);
}
}

QWidget *MainWindow::makeAppearanceSettings()
{
    auto *appearance = new QWidget;
    appearance->setObjectName("appearancePage");
    auto *center = new QHBoxLayout(appearance);
    center->setContentsMargins(0, 0, 0, 0);
    auto *panel = new QWidget;
    panel->setMaximumWidth(850);
    center->addStretch();
    center->addWidget(panel, 100, Qt::AlignTop);
    center->addStretch();
    auto *layout = new QVBoxLayout(panel);
    layout->setContentsMargins(2, 2, 2, 16);
    layout->setSpacing(14);

    auto *header = new QHBoxLayout;
    header->addWidget(live::text(label({}, "heading"), [] { return tr("Appearance"); }), 1);
    appearanceReset_ = live::text(new QPushButton, [] { return tr("Reset default"); });
    appearanceReset_->setObjectName("resetAppearanceDefault");
    appearanceReset_->setProperty("actionRole", "secondary");
    appearanceReset_->hide();
    header->addWidget(appearanceReset_);
    layout->addLayout(header);
    layout->addSpacing(10);
    layout->addWidget(live::text(label({}, "sectionTitle"), [] { return tr("Visual style"); }));

    auto *modePanel = settingsGroup();
    auto *mode = new QFormLayout(modePanel);
    mode->setContentsMargins(16, 12, 16, 12);
    mode->setRowWrapPolicy(QFormLayout::WrapLongRows);
    mode->setFieldGrowthPolicy(QFormLayout::AllNonFixedFieldsGrow);
    mode->setLabelAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    auto *tiles = new QWidget;
    auto *previews = new QHBoxLayout(tiles);
    previews->setContentsMargins(0, 0, 0, 0);
    previews->setSpacing(8);
    previews->addStretch();
    themeChoice_ = new QButtonGroup(appearance);
    const auto addMode = [this, previews](live::Phrase title, const QString &key, const QString &name) {
        auto *tile = new ThemeTile({}, key, name);
        live::bind(tile, [tile, title = std::move(title)] {
            const auto label = title();
            tile->setText(label);
            tile->setAccessibleName(label);
        });
        themeChoice_->addButton(tile);
        previews->addWidget(tile);
    };
    addMode([] { return tr("System"); }, "system", "themeSystem");
    addMode([] { return tr("Light"); }, "light", "themeLight");
    addMode([] { return tr("Dark"); }, "dark", "themeDark");
    selectTheme(*themeChoice_, defaultAppearance.value("theme").toString());
    mode->addRow(live::text(label({}, "sectionTitle"), [] { return tr("Mode"); }), tiles);
    layout->addWidget(modePanel);

    const auto makeCard = [](QVBoxLayout &destination) {
        auto *card = settingsGroup();
        auto *rows = new QGridLayout(card);
        rows->setContentsMargins(16, 10, 16, 10);
        rows->setHorizontalSpacing(12);
        rows->setVerticalSpacing(8);
        rows->setColumnStretch(0, 1);
        destination.addWidget(card);
        return rows;
    };
    auto *visual = makeCard(*layout);
    const auto colorChange = [this] {
        applyAppearance();
        schedulePreferencesSave();
    };
    accentChoice_ = new QComboBox;
    accentChoice_->setObjectName("accentChoice");
    live::text(accentChoice_, [] { return tr("Accent"); }, &QWidget::setAccessibleName);
    accentChoice_->setIconSize({16, 16});
    accentChoice_->setItemDelegate(new AppearanceChoiceDelegate(accentChoice_));
    live::item(accentChoice_, [] { return tr("Default teal"); }, "cirnet");
    live::item(accentChoice_, [] { return tr("Cobalt blue"); }, "cobalt");
    live::item(accentChoice_, [] { return tr("Green"); }, "green");
    live::item(accentChoice_, [] { return tr("Yellow"); }, "yellow");
    live::item(accentChoice_, [] { return tr("Pink"); }, "pink");
    live::item(accentChoice_, [] { return tr("Orange"); }, "orange");
    live::item(accentChoice_, [] { return tr("Purple"); }, "purple");
    live::item(accentChoice_, [] { return tr("White"); }, "white");
    live::item(accentChoice_, [] { return tr("Custom"); }, "custom");
    accentColor_ = new AppearanceColorField("appearanceAccentColor", [] { return tr("Custom accent"); },
                                            QPalette::Highlight, false, colorChange);
    accentColor_->hide();
    auto *accentRow = new QWidget;
    auto *accentLayout = new QHBoxLayout(accentRow);
    accentLayout->setContentsMargins(0, 0, 0, 0);
    accentLayout->setSpacing(6);
    accentLayout->addWidget(accentChoice_);
    accentLayout->addWidget(accentColor_);
    accentRow->setFocusProxy(accentChoice_);
    accentRow->setFocusPolicy(Qt::StrongFocus);
    connect(accentChoice_, &QComboBox::currentIndexChanged, appearance, [this] {
        const bool custom = accentChoice_->currentData() == "custom";
        if (custom && accentColor_->stored().isEmpty())
            accentColor_->load(QApplication::palette().color(QPalette::Highlight).name());
        accentColor_->setVisible(custom);
    });
    backgroundColor_ = new AppearanceColorField("appearanceBackground", [] { return tr("Background"); },
                                                QPalette::Window, true, colorChange);
    foregroundColor_ = new AppearanceColorField("appearanceForeground", [] { return tr("Foreground"); },
                                                QPalette::WindowText, true, colorChange);
    auto *contrast = live::text(label({},
                           "meta"), [] { return tr("Low contrast between background and text may make some content difficult to read."); });
    contrast->setObjectName("appearanceContrastWarning");
    const auto refreshColors = [this, contrast] {
        const auto palette = QApplication::palette();
        const auto background = palette.color(QPalette::Window);
        const auto foreground = palette.color(QPalette::WindowText);
        const bool light = appearanceColorContrast(background, Qt::black) >= appearanceColorContrast(background, Qt::white);
        for (int i = 0; i < accentChoice_->count(); ++i) {
            const auto key = accentChoice_->itemData(i).toString();
            const auto custom = accentColor_->stored().isEmpty() ? palette.color(QPalette::Highlight).name()
                                                                 : accentColor_->stored();
            accentChoice_->setItemIcon(i, colorDot(appearanceAccent(key, light, custom)));
        }
        for (auto *field : {backgroundColor_, foregroundColor_, accentColor_})
            field->refresh();
        const bool lowContrast = appearanceColorContrast(background, foreground) < 4.5;
        if (lowContrast)
            contrast->setStyleSheet(light ? QStringLiteral("color: #000000") : QStringLiteral("color: #ffffff"));
        contrast->setVisible(lowContrast);
    };
    connect(theme_, &ThemeManager::changed, appearance, refreshColors);
    uiFontChoice_ = new QComboBox;
    uiFontChoice_->setObjectName("uiFontChoice");
    live::text(uiFontChoice_, [] { return tr("Interface font"); }, &QWidget::setAccessibleName);
    live::item(uiFontChoice_, [] { return tr("System"); }, "system");
    live::item(uiFontChoice_, [] { return tr("Monospace"); }, "monospace");
    uiFontChoice_->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
    uiFontChoice_->setMinimumContentsLength(12);
    uiFontChoice_->setMaxVisibleItems(12);
    uiFontChoice_->view()->setMinimumWidth(248);
    uiFontChoice_->setItemDelegate(new AppearanceChoiceDelegate(uiFontChoice_));
    for (const auto &family : QFontDatabase::families()) {
        if (family.startsWith(u'@') || !QFontDatabase::isSmoothlyScalable(family))
            continue;
        uiFontChoice_->addItem(family);
        uiFontChoice_->setItemData(uiFontChoice_->count() - 1, family, installedFontRole);
        uiFontChoice_->setItemData(uiFontChoice_->count() - 1, family, Qt::ToolTipRole);
    }
    addSettingRow(*visual, 0, [] { return tr("&Accent"); }, {}, accentRow);
    addSettingRow(*visual, 1, [] { return tr("Background"); }, {}, backgroundColor_);
    addSettingRow(*visual, 2, [] { return tr("Foreground"); }, {}, foregroundColor_);
    addSettingRow(*visual, 3, [] { return tr("Interface &font"); }, {}, uiFontChoice_);
    visual->addWidget(contrast, 7, 0, 1, 2);
    refreshColors();

    appearanceAdvancedToggle_ = new QToolButton;
    appearanceAdvancedToggle_->setObjectName("appearanceAdvancedToggle");
    live::text(appearanceAdvancedToggle_, [] { return tr("Advanced"); });
    live::text(appearanceAdvancedToggle_, [] { return tr("Show or hide advanced appearance settings."); }, &QWidget::setAccessibleDescription);
    appearanceAdvancedToggle_->setCheckable(true);
    appearanceAdvancedToggle_->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    appearanceAdvancedToggle_->setArrowType(Qt::RightArrow);
    appearanceAdvancedToggle_->setFocusPolicy(Qt::StrongFocus);
    layout->addSpacing(4);
    layout->addWidget(appearanceAdvancedToggle_, 0, Qt::AlignLeft);

    auto *advanced = new QWidget;
    auto *advancedLayout = new QVBoxLayout(advanced);
    advancedLayout->setContentsMargins(0, 0, 0, 0);
    advancedLayout->setSpacing(14);
    auto *typography = makeCard(*advancedLayout);
    const auto pixelControl = [](const char *name, live::Phrase title, int minimum, int maximum, int value) {
        auto *control = new QSpinBox;
        control->setObjectName(QLatin1StringView(name));
        live::text(control, std::move(title), &QWidget::setAccessibleName);
        control->setRange(minimum, maximum);
        control->setValue(value);
        control->setSuffix(QStringLiteral(" px"));
        return control;
    };
    textScale_ = pixelControl("textScale", [] { return tr("Interface text size"); }, 10, 26, baseTextPixels);
    contentScaleChoice_ = pixelControl("contentScaleChoice", [] { return tr("Conversation text size"); }, 10, 26, baseTextPixels);
    codeScaleChoice_ = pixelControl("codeScaleChoice", [] { return tr("Code text size"); }, 10, 26, baseTextPixels);
    addSettingRow(*typography, 0, [] { return tr("&Interface text size"); },
                  [] { return tr("Scale text throughout the desktop."); }, textScale_);
    addSettingRow(*typography, 1, [] { return tr("&Conversation text size"); },
                  [] { return tr("Scale the transcript and composer independently."); }, contentScaleChoice_);
    addSettingRow(*typography, 2, [] { return tr("Code text si&ze"); },
                  [] { return tr("Scale monospaced diagnostics and structured details."); }, codeScaleChoice_);

    auto *interaction = makeCard(*advancedLayout);
    densityChoice_ = pixelControl("densityChoice", [] { return tr("Density"); }, 2, 14, baseDensityPixels);
    languageChoice_ = new QComboBox;
    languageChoice_->setObjectName("languageChoice");
    live::text(languageChoice_, [] { return tr("Language"); }, &QWidget::setAccessibleName);
    languageChoice_->addItem(QStringLiteral("English"), "en");
    languageChoice_->addItem(QStringLiteral("Tiếng Việt"), "vi");
    languageChoice_->addItem(QStringLiteral("简体中文"), "zh_CN");
    reducedMotion_ = new QCheckBox;
    reducedMotion_->setObjectName("reducedMotion");
    live::text(reducedMotion_, [] { return tr("Reduced motion"); }, &QWidget::setAccessibleName);
    addSettingRow(*interaction, 0, [] { return tr("&Density"); }, [] { return tr("Adjust the spacing between controls."); }, densityChoice_);
    addSettingRow(*interaction, 1, [] { return tr("Language"); }, [] { return tr("Changes the language of the open Dashboard."); }, languageChoice_);
    addSettingRow(*interaction, 2, [] { return tr("&Reduced motion"); },
                  [] { return tr("Skip motion effects while keeping visual feedback."); }, reducedMotion_);
    advanced->hide();
    layout->addWidget(advanced);
    connect(appearanceAdvancedToggle_, &QToolButton::toggled, advanced, [this, advanced](bool expanded) {
        appearanceAdvancedToggle_->setArrowType(expanded ? Qt::DownArrow : Qt::RightArrow);
        advanced->setVisible(expanded);
    });

    connect(appearanceReset_, &QPushButton::clicked, this, [this] {
        applyAppearanceControls(defaultAppearance);
        followSavedLanguage_ = true;
        live::language().set(languageChoice_->currentData().toString());
        applyAppearance();
        schedulePreferencesSave();
    });
    return scrollPanel(appearance);
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
    categories->addWidget(live::text(label({}, "heading"), [] { return tr("Settings"); }));
    settingsSearch_ = new QLineEdit;
    settingsSearch_->setObjectName("settingsSearch");
    settingsSearch_->setProperty("searchField", true);
    live::placeholder(settingsSearch_, [] { return tr("Search settings"); });
    live::text(settingsSearch_, [] { return tr("Search settings"); }, &QWidget::setAccessibleName);
    settingsSearch_->setClearButtonEnabled(true);
    categories->addWidget(settingsSearch_);
    settingsNavigation_ = new QListWidget;
    settingsNavigation_->setObjectName("settingsNavigation");
    live::text(settingsNavigation_, [] { return tr("Settings section"); }, &QWidget::setAccessibleName);
    settingsNavigation_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    settingsNavigation_->setTextElideMode(Qt::ElideRight);
    categories->addWidget(settingsNavigation_, 1);
    auto *noResults = live::text(label({}, "meta"), [] { return tr("No matching settings."); });
    noResults->hide();
    categories->addWidget(noResults);
    auto *back = live::text(new QPushButton, [] { return tr("Back"); });
    back->setObjectName("settingsBack");
    back->setIcon(uiIcon(UiIcon::Back));
    back->setProperty("actionRole", "secondary");
    live::text(back, [] { return tr("Back to workspace"); }, &QWidget::setAccessibleName);
    live::text(back, [] { return tr("Back to workspace"); }, &QWidget::setToolTip);
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
    settingsNavigation_->setIconSize(QSize(17, 17));
    const auto section = [this](live::Phrase title, const QString &key, live::Phrase keywords, QWidget *panel) {
        static const QHash<QString, UiIcon> icons{
            {"appearance", UiIcon::Appearance}, {"repositories", UiIcon::Folder},
            {"runtime", UiIcon::Runtime}, {"configuration", UiIcon::Sliders},
            {"storage", UiIcon::Storage}, {"archived", UiIcon::Archive},
            {"notifications", UiIcon::Bell}, {"about", UiIcon::Info}
        };
        auto *item = new QListWidgetItem(uiIcon(icons.value(key, UiIcon::Settings)), {}, settingsNavigation_);
        item->setData(Qt::UserRole, key);
        live::bind(settingsNavigation_, [item, title = std::move(title), keywords = std::move(keywords)] {
            const auto label = title();
            item->setText(label);
            item->setData(Qt::UserRole + 1, label + ' ' + keywords());
            item->setToolTip(label);
        });
        settingsPages_->addWidget(panel);
    };

    const auto appearanceSearchTerms = [] { return QStringList{
        tr("Theme mode"), tr("Density"), tr("Text size"), tr("Interface text size"),
        tr("Conversation text size"), tr("Code text size"), tr("Reduced motion"),
        tr("System"), tr("Light"), tr("Dark"), tr("Accent"), tr("Interface font"),
        tr("Advanced"), tr("Background"), tr("Foreground"), tr("Reset default"), tr("Language"),
    }.join(u' '); };
    section([] { return tr("Appearance"); }, "appearance", appearanceSearchTerms, makeAppearanceSettings());
    section([] { return tr("Repositories"); }, "repositories", [] { return tr("binding folder path worktree relink archive"); }, repositoriesPage_);

    auto *runtime = new QWidget;
    auto *runtimeLayout = new QVBoxLayout(runtime);
    runtimeLayout->setContentsMargins(2, 8, 2, 8);
    runtimeLayout->addWidget(live::text(label({}, "heading"), [] { return tr("Agents & Runtime"); }));
    runtimeStatus_ = label({}, "runtimeState");
    runtimeCapabilities_ = label({}, "runtimeCapabilities");
    runtimeLayout->addWidget(runtimeStatus_);
    runtimeLayout->addWidget(runtimeCapabilities_);
    auto *runtimeActions = new QHBoxLayout;
    reconnect_ = live::text(new QPushButton, [] { return tr("Reconnect"); });
    reconnect_->setObjectName("reconnectRuntime");
    runtimeActions->addWidget(reconnect_);
    connect(reconnect_, &QPushButton::clicked, this, &MainWindow::reconnectRuntime);
    auto *diagnostics = live::text(new QPushButton, [] { return tr("Diagnostics"); });
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
        text->setProperty("codeText", true);
        text->setReadOnly(true);
        const auto state = online_ ? tr("Connected") : connecting_ ? tr("Connecting") : tr("Disconnected");
        text->setPlainText(tr("%1 %2\nRuntime: %3\n%4")
                              .arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME), QStringLiteral(CIRNETRULES_APP_VERSION),
                                   state, runtimeCapabilities_->text()));
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

    auto *controlsPanel = settingsGroup();
    auto *configurationRows = new QGridLayout(controlsPanel);
    configurationRows->setColumnStretch(0, 1);
    auto *agentInspect = new QComboBox;
    agentInspect->setObjectName("configurationAdapter");
    live::text(agentInspect, [] { return tr("Inspect adapter"); }, &QWidget::setAccessibleName);
    agentInspect->setSizeAdjustPolicy(QComboBox::AdjustToContents);
    agentInspect->setMinimumContentsLength(14);
    addSettingRow(*configurationRows, 0, [] { return tr("Inspect &adapter"); },
                  [] { return tr("View the capability reported by the local runtime."); }, agentInspect);
    auto *capabilityDetails = label({}, "configurationCapability");
    capabilityDetails->setTextInteractionFlags(Qt::TextSelectableByMouse);
    configurationRows->addWidget(capabilityDetails, 1, 0, 1, 2);
    auto *limits = label({}, "configurationLimits");
    limits->setTextInteractionFlags(Qt::TextSelectableByMouse);
    configurationRows->addWidget(limits, 2, 0, 1, 2);
    runtimeLayout->insertWidget(runtimeLayout->count() - 1, controlsPanel);
    connect(agentInspect, &QComboBox::currentIndexChanged, this, [this, capabilityDetails, agentInspect] {
        const auto native = runtimeInfo_.value("native_agent").toObject();
        const auto adapter = agentInspect->currentText();
        capabilityDetails->setText(tr("Requested: Local handoff\nEffective: %1\nSource: local runtime")
            .arg(online_ ? uiStatusLabel(native.value(adapter).toString(tr("Unavailable"))) : tr("Unavailable")));
    });
    section([] { return tr("Agents & Runtime"); }, "runtime",
            [] { return tr("connection reconnect diagnostics capability"); }, scrollPanel(runtime));

    const SettingsRequest request = [this](const QString &method, QJsonObject params, RuntimeClient::Reply reply) {
        call(method, std::move(params), std::move(reply));
    };
    governance_ = new GovernancePanel(request, [this] {
        return repositories_;
    });
    section([] { return tr("Configuration"); }, "configuration",
            [] { return tr("governance library rules skills profiles references policies"); }, governance_);

    section([] { return tr("Storage"); }, "storage", [] { return tr("chat message usage backup verify compact data folder"); },
            scrollPanel(makeStorageSettings(client_, *this, request)));
    section([] { return tr("Archived chats"); }, "archived", [] { return tr("conversation search unarchive restore delete"); },
            makeArchivedChats(request, [this] { return repositories_; }, [this] { refreshConversations(); },
            [this](const QString &id, std::function<void(bool)> complete) {
                deleteConversationData(id, {}, std::move(complete));
            }));

    auto *notices = new QWidget;
    auto *noticesLayout = new QVBoxLayout(notices);
    noticesLayout->setContentsMargins(2, 8, 2, 8);
    noticesLayout->addWidget(live::text(label({}, "heading"), [] { return tr("Notifications"); }));
    auto *noticeControls = new QHBoxLayout;
    notificationChoice_ = new QComboBox;
    notificationChoice_->setObjectName("notificationLevel");
    live::text(notificationChoice_, [] { return tr("Toast level"); }, &QWidget::setAccessibleName);
    live::item(notificationChoice_, [] { return tr("All toasts"); }, "all");
    live::item(notificationChoice_, [] { return tr("Warnings and errors"); }, "important");
    noticeControls->addWidget(notificationChoice_);
    auto *clear = live::text(new QPushButton, [] { return tr("Clear read"); });
    clear->setObjectName("clearNotifications");
    noticeControls->addWidget(clear);
    noticeControls->addStretch();
    noticesLayout->addLayout(noticeControls);
    auto *history = new QListWidget;
    history->setObjectName("notificationHistory");
    live::text(history, [] { return tr("Notification history"); }, &QWidget::setAccessibleName);
    history->setWordWrap(true);
    noticesLayout->addWidget(history, 1);
    notifications_->attachHistory(*history);
    connect(clear, &QPushButton::clicked, notifications_, &NotificationCenter::clearRead);
    section([] { return tr("Notifications"); }, "notifications", [] { return tr("toast history warnings errors"); }, notices);

    auto *about = new QWidget;
    about->setObjectName("aboutPage");
    auto *aboutLayout = new QVBoxLayout(about);
    aboutLayout->setContentsMargins(2, 8, 2, 8);
    aboutLayout->addWidget(live::text(label({}, "heading"), [] { return tr("About %1").arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME)); }));
    aboutLayout->addWidget(live::text(label({}, "lead"), [] { return QStringLiteral(CIRNETRULES_PRODUCT_NAME); }));
    aboutDetails_ = label({}, "aboutDetails");
    aboutDetails_->setTextInteractionFlags(Qt::TextSelectableByMouse);
    aboutLayout->addWidget(aboutDetails_);
    aboutLayout->addStretch();
    section([] { return tr("About"); }, "about", [] { return tr("version release application identity"); }, scrollPanel(about));

    auto *saveRow = new QHBoxLayout;
    preferencesStatus_ = label({}, "preferencesStatus");
    preferencesStatus_->hide();
    saveRow->addWidget(preferencesStatus_, 1);
    preferencesRetry_ = live::text(new QPushButton, [] { return tr("Retry saving"); });
    preferencesRetry_->setObjectName("preferencesRetry");
    preferencesRetry_->hide();
    saveRow->addWidget(preferencesRetry_);
    auto *preferencesRevert = live::text(new QPushButton, [] { return tr("Revert unsaved changes"); });
    preferencesRevert->setObjectName("preferencesRevert");
    preferencesRevert->hide();
    saveRow->addWidget(preferencesRevert);
    layout->addLayout(saveRow);
    connect(preferencesRevert, &QPushButton::clicked, this, [this, preferencesRevert] {
        if (preferencesSaving_ || preferencesLoading_)
            return;
        preferencesTimer_->stop();
        preferencesDirty_ = false;
        preferencesRetryRequired_ = false;
        applyPreferences(savedPreferences_);
        preferencesStatus_->hide();
        preferencesRetry_->hide();
        preferencesRevert->hide();
        loadPreferences();
    });
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
    const auto fitCategories = [this, sidebar] {
        const auto metrics = settingsNavigation_->fontMetrics();
        int width = 200;
        for (int row = 0; row < settingsNavigation_->count(); ++row)
            width = qMax(width, metrics.horizontalAdvance(settingsNavigation_->item(row)->text()) + 76);
        sidebar->setFixedWidth(qMin(width, 300));
    };
    fitCategories();
    connect(theme_, &ThemeManager::changed, sidebar, fitCategories);
    connect(&live::language(), &live::Language::changed, sidebar, fitCategories);
    const auto filterCategories = [this, noResults](const QString &text) {
        bool matched = false;
        for (int row = 0; row < settingsNavigation_->count(); ++row) {
            auto *item = settingsNavigation_->item(row);
            const bool visible = item->data(Qt::UserRole + 1).toString().contains(text.trimmed(), Qt::CaseInsensitive);
            item->setHidden(!visible);
            matched |= visible;
        }
        noResults->setVisible(!matched);
    };
    connect(settingsSearch_, &QLineEdit::textChanged, this, filterCategories);
    connect(&live::language(), &live::Language::changed, page, [this, filterCategories] {
        filterCategories(settingsSearch_->text());
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
            const auto matches = [&query](const QString &text) { return text.contains(query, Qt::CaseInsensitive); };
            if (matches(tr("Reduced motion")))
                target = reducedMotion_;
            else if (matches(tr("Text size")) || matches(tr("Interface text size")))
                target = textScale_;
            else if (matches(tr("Conversation text size")))
                target = contentScaleChoice_;
            else if (matches(tr("Code text size")))
                target = codeScaleChoice_;
            else if (matches(tr("Density")))
                target = densityChoice_;
            else if (matches(tr("Language")))
                target = languageChoice_;
            else if (matches(tr("Accent")))
                target = accentChoice_;
            else if (matches(tr("Background")))
                target = backgroundColor_->editor();
            else if (matches(tr("Foreground")))
                target = foregroundColor_->editor();
            else if (matches(tr("Interface font")))
                target = uiFontChoice_;
            else if (matches(tr("Advanced")))
                target = appearanceAdvancedToggle_;
            else if (matches(tr("Theme mode"))) {
                if (auto *selected = themeChoice_->checkedButton())
                    target = selected;
            }
            if (target == reducedMotion_ || target == textScale_ || target == contentScaleChoice_ ||
                target == codeScaleChoice_ || target == densityChoice_ || target == languageChoice_)
                appearanceAdvancedToggle_->setChecked(true);
        }
        target->setFocus(Qt::ShortcutFocusReason);
        if (target != settingsNavigation_) {
            auto *scroll = qobject_cast<QScrollArea *>(settingsPages_->currentWidget());
            if (scroll) {
                QTimer::singleShot(0, scroll, [scroll, target] {
                    scroll->ensureWidgetVisible(target, 12, 12);
                });
            }
        }
    });
    const auto visualChange = [this] {
        applyAppearance();
        schedulePreferencesSave();
    };
    auto *previewTimer = new QTimer(page);
    previewTimer->setSingleShot(true);
    previewTimer->setInterval(45);
    connect(previewTimer, &QTimer::timeout, this, &MainWindow::applyAppearance);
    const auto queuePreview = [this, previewTimer] {
        previewTimer->start();
        schedulePreferencesSave();
    };
    connect(themeChoice_, &QButtonGroup::buttonToggled, this, [visualChange](QAbstractButton *, bool checked) {
        if (checked)
            visualChange();
    });
    for (auto *control : {textScale_, contentScaleChoice_, codeScaleChoice_, densityChoice_})
        connect(control, &QSpinBox::valueChanged, this, queuePreview);
    for (auto *control : {accentChoice_, uiFontChoice_})
        connect(control, &QComboBox::currentIndexChanged, this, queuePreview);
    connect(languageChoice_, &QComboBox::currentIndexChanged, this, [this, visualChange] {
        followSavedLanguage_ = true;
        live::language().set(languageChoice_->currentData().toString());
        visualChange();
    });
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
    governance_->setAvailable(online_);
    preferencesRetry_->setEnabled(online_);
    QStringList adapters;
    const auto native = online_ ? runtimeInfo_.value("native_agent").toObject() : QJsonObject{};
    for (auto it = native.begin(); it != native.end(); ++it)
        adapters.append(it.key() + QStringLiteral(" · ") + uiStatusLabel(it.value().toString()));
    auto capability = tr("Native execution capability: %1")
                      .arg(adapters.isEmpty() ? tr("Unavailable") : adapters.join(QStringLiteral("\n")));
    const auto controls = online_ ? runtimeInfo_.value("execution_enforcement").toObject() : QJsonObject{};
    if (!controls.isEmpty()) {
        QStringList details;
        for (auto it = controls.begin(); it != controls.end(); ++it)
            details.append(uiStatusLabel(it.key()) + QStringLiteral(" · ") + uiStatusLabel(it.value().toString()));
        capability += QStringLiteral("\n\n") + tr("Execution controls: %1").arg(details.join(QStringLiteral("\n")));
    }
    runtimeCapabilities_->setText(capability);
    auto *agentInspect = settingsPage_->findChild<QComboBox *>("configurationAdapter");
    auto *capabilityDetails = settingsPage_->findChild<QLabel *>("configurationCapability");
    const QSignalBlocker inspectSignals(agentInspect);
    const auto selectedAdapter = agentInspect->currentText();
    agentInspect->clear();
    agentInspect->addItems(native.keys());
    agentInspect->setCurrentText(selectedAdapter);
    agentInspect->setEnabled(!native.isEmpty());
    capabilityDetails->setText(tr("Requested: Local handoff\nEffective: %1\nSource: local runtime")
        .arg(uiStatusLabel(native.value(agentInspect->currentText()).toString(tr("Unavailable")))));
    const auto storageLimits = online_ ? runtimeInfo_.value("storage_jobs").toObject() : QJsonObject{};
    auto *limits = settingsPage_->findChild<QLabel *>("configurationLimits");
    limits->setText(storageLimits.isEmpty() ? tr("Storage worker limits are unavailable.")
        : tr("Storage: %1 worker(s), %2 admitted operations, %3 retained results, %4 s execution budget.")
            .arg(storageLimits.value("workers").toInt()).arg(storageLimits.value("active_limit").toInt())
            .arg(storageLimits.value("retained_limit").toInt()).arg(storageLimits.value("deadline_seconds").toInt()));
    aboutDetails_->setText(tr("%1\nApplication %2\nRelease %3\n\n%4")
                              .arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME), QStringLiteral(CIRNETRULES_APP_VERSION),
                                   QStringLiteral(CIRNETRULES_RELEASE_NAME), capability));
}

QJsonObject MainWindow::appearanceSettings() const
{
    const auto fontFamily = uiFontChoice_->currentData(installedFontRole).toString();
    QJsonObject style{
        {"accent", accentChoice_->currentData().toString()},
        {"ui_font", fontFamily.isEmpty() ? uiFontChoice_->currentData().toString() : QStringLiteral("system")},
        {"interface_px", textScale_->value()}, {"conversation_px", contentScaleChoice_->value()},
        {"code_px", codeScaleChoice_->value()}, {"density_px", densityChoice_->value()},
        {"language", languageChoice_->currentData().toString()}
    };
    if (!fontFamily.isEmpty())
        style.insert("font_family", fontFamily);
    if (!accentColor_->stored().isEmpty())
        style.insert("custom_accent", accentColor_->stored());
    if (!backgroundColor_->stored().isEmpty())
        style.insert("background", backgroundColor_->stored());
    if (!foregroundColor_->stored().isEmpty())
        style.insert("foreground", foregroundColor_->stored());
    return normalizedAppearance({{"theme", themeMode(*themeChoice_)},
                                 {"reduced_motion", reducedMotion_->isChecked()},
                                 {"appearance", style}});
}

QJsonObject MainWindow::preferenceSettings() const
{
    auto settings = appearanceSettings();
    const auto repoLayout = repositoryLayout();
    settings.insert("sidebar_width", sidebarWidth_);
    settings.insert("sidebar_collapsed", sidebarCollapsed_);
    settings.insert("repository_layout", repoLayout.isEmpty() ? QJsonValue(QJsonValue::Null) : QJsonValue(repoLayout));
    settings.insert("notification_level", notificationChoice_->currentData().toString());
    return settings;
}

void MainWindow::applyAppearance()
{
    const auto visual = appearanceSettings();
    theme_->apply(visual.value("theme").toString(), visual.value("density").toString(),
                  visual.value("text_scale").toInt(), visual.value("appearance").toObject());
    appearanceReset_->setVisible(visual != defaultAppearance);
    notifications_->setLevel(notificationChoice_->currentData().toString());
    notifications_->setReducedMotion(reducedMotion_->isChecked());
}

void MainWindow::applyAppearanceControls(const QJsonObject &settings)
{
    const auto normalized = normalizedAppearance(settings);
    const auto appearance = normalized.value("appearance").toObject();
    const QSignalBlocker themeSignals(themeChoice_);
    const QSignalBlocker densitySignals(densityChoice_);
    const QSignalBlocker scaleSignals(textScale_);
    const QSignalBlocker motionSignals(reducedMotion_);
    const QSignalBlocker accentSignals(accentChoice_);
    const QSignalBlocker fontSignals(uiFontChoice_);
    const QSignalBlocker contentSignals(contentScaleChoice_);
    const QSignalBlocker codeSignals(codeScaleChoice_);
    const QSignalBlocker languageSignals(languageChoice_);
    selectTheme(*themeChoice_, normalized.value("theme").toString());
    textScale_->setValue(appearance.value("interface_px").toInt());
    contentScaleChoice_->setValue(appearance.value("conversation_px").toInt());
    codeScaleChoice_->setValue(appearance.value("code_px").toInt());
    densityChoice_->setValue(appearance.value("density_px").toInt());
    reducedMotion_->setChecked(normalized.value("reduced_motion").toBool());
    selectData(languageChoice_, appearance.value("language").toString());
    selectData(accentChoice_, appearance.value("accent").toString());
    accentColor_->load(appearance.value("custom_accent").toString());
    accentColor_->setVisible(accentChoice_->currentData() == "custom");
    backgroundColor_->load(appearance.value("background").toString());
    foregroundColor_->load(appearance.value("foreground").toString());
    const auto family = appearance.value("font_family").toString();
    if (family.isEmpty()) {
        selectData(uiFontChoice_, appearance.value("ui_font").toString());
    } else {
        int index = uiFontChoice_->findData(family, installedFontRole);
        if (index < 0) {
            uiFontChoice_->addItem(tr("Unavailable: %1").arg(family));
            index = uiFontChoice_->count() - 1;
            uiFontChoice_->setItemData(index, family, installedFontRole);
        }
        uiFontChoice_->setCurrentIndex(index);
    }
}

void MainWindow::applyPreferences(const QJsonObject &settings)
{
    applyingPreferences_ = true;
    applyAppearanceControls(settings);
    if (followSavedLanguage_)
        live::language().set(languageChoice_->currentData().toString());
    const QSignalBlocker notificationSignals(notificationChoice_);
    selectData(notificationChoice_, settings.value("notification_level").toString("all"));
    sidebarWidth_ = qBound(210, settings.value("sidebar_width").toInt(260), 360);
    sidebarCollapsed_ = settings.value("sidebar_collapsed").toBool();
    applyAppearance();
    changeSidebar(sidebarCollapsed_);
    restoreRepositoryLayout(settings.value("repository_layout").toObject());
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
            setPreferenceRevert(settingsPage_, preferencesDirty_);
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
            const auto beforeVisual = normalizedAppearance(savedPreferences_);
            for (auto it = local.begin(); it != local.end(); ++it) {
                if (it.key() == "appearance") {
                    auto style = normalizedAppearance(remote).value("appearance").toObject();
                    const auto before = beforeVisual.value("appearance").toObject();
                    const auto pending = it.value().toObject();
                    for (auto part = before.begin(); part != before.end(); ++part) {
                        if (!pending.contains(part.key()))
                            style.remove(part.key());
                    }
                    for (auto part = pending.begin(); part != pending.end(); ++part) {
                        if (part.value() != before.value(part.key()))
                            style.insert(part.key(), part.value());
                    }
                    merged.insert("appearance", style);
                } else if (it.value() != (beforeVisual.contains(it.key())
                               ? beforeVisual.value(it.key()) : savedPreferences_.value(it.key()))) {
                    merged.insert(it.key(), it.value());
                }
            }
        }
        preferencesLoaded_ = true;
        preferencesRevision_ = revision;
        if (stale) {
            preferencesRetryRequired_ = true;
            preferencesStatus_->setText(tr("Preferences changed in another window. Review or retry your pending changes."));
            preferencesStatus_->show();
            preferencesRetry_->show();
            setPreferenceRevert(settingsPage_, true);
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
        setPreferenceRevert(settingsPage_, false);
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
        setPreferenceRevert(settingsPage_, true);
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
            setPreferenceRevert(settingsPage_, true);
            showStatus(preferencesStatus_->text(), true);
            return;
        }
        preferencesRevision_ = result.value("revision").toInteger();
        savedPreferences_ = result.value("settings").toObject();
        preferencesDirty_ = preferenceSettings() != settings;
        preferencesStatus_->hide();
        preferencesRetry_->hide();
        setPreferenceRevert(settingsPage_, false);
        if (preferencesDirty_) {
            preferencesTimer_->start();
        } else if (closeAfterSave_) {
            closeAfterSave_ = false;
            stopRuntimeAndClose();
        }
    });
}
