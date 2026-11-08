#pragma once

#include <QColor>
#include <QIcon>
#include <QObject>

class QApplication;

enum class UiIcon { Add, Automations, Collapse, Expand, Settings, Status, Close, Info, Warning, Success };

QIcon uiIcon(UiIcon symbol, const QColor &color = {});
void applyTheme(QApplication &app);

class ThemeManager final : public QObject {
    Q_OBJECT
public:
    explicit ThemeManager(QObject *parent = nullptr);
    void apply(const QString &theme, const QString &density, int textScale);

signals:
    void changed();

private:
    QString theme_;
    QString density_;
    int textScale_ = 100;
    bool light_ = false;
};
