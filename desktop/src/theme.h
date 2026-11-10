#pragma once

#include <QColor>
#include <QIcon>
#include <QJsonObject>
#include <QObject>

class QApplication;

enum class UiIcon {
    Add, Automations, Collapse, Expand, Settings, Status, Close, Clear, Refresh,
    Info, Warning, Success, Error, Question, Search, Back, Appearance, Folder,
    Sliders, Storage, Archive, Runtime, Bell, Trash
};

enum class UiTone { TableHover, TableSelection, Info, Success, Warning, Danger };

QIcon uiIcon(UiIcon symbol, const QColor &color = {});
QIcon uiPrimaryIcon(UiIcon symbol);
QColor uiTone(UiTone tone);
QColor defaultAppearanceBackground(bool light);
QColor appearanceAccent(const QString &accent, bool light, const QString &custom = {});
double appearanceColorContrast(const QColor &first, const QColor &second);
inline QString uiStatusLabel(QString code) { return code.replace(u'_', u' '); }
void applyTheme(QApplication &app);

class ThemeManager final : public QObject {
    Q_OBJECT
public:
    explicit ThemeManager(QObject *parent = nullptr);
    void apply(const QString &theme, const QString &density, int textScale,
               const QJsonObject &appearance = {});

signals:
    void changed();

private:
    QString theme_;
    QString density_;
    int textScale_ = 100;
    QJsonObject appearance_;
    bool light_ = false;
};
