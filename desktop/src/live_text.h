#pragma once

#include <QComboBox>
#include <QObject>
#include <QSignalBlocker>
#include <QString>

#include <functional>
#include <memory>
#include <utility>

class QTranslator;
class QLabel;

namespace live {
class Language final : public QObject {
    Q_OBJECT
public:
    explicit Language(QObject *parent);
    ~Language() override;
    bool set(QString locale);

signals:
    void changed();

private:
    QString locale_ = QStringLiteral("en");
    std::unique_ptr<QTranslator> translator_;
};

Language &language();
using Phrase = std::function<QString()>;
void message(QLabel *label, Phrase phrase);

template<class Update> void bind(QObject *owner, Update update)
{
    update();
    QObject::connect(&language(), &Language::changed, owner, std::move(update));
}

template<class Widget, class Setter>
Widget *text(Widget *widget, Phrase phrase, Setter setter)
{
    bind(widget, [widget, phrase = std::move(phrase), setter] { std::invoke(setter, widget, phrase()); });
    return widget;
}

template<class Widget> Widget *text(Widget *widget, Phrase phrase)
{
    return text(widget, std::move(phrase), &Widget::setText);
}

template<class Widget> void placeholder(Widget *widget, Phrase phrase)
{
    text(widget, std::move(phrase), &Widget::setPlaceholderText);
}

inline void item(QComboBox *combo, Phrase phrase, const QVariant &data = {})
{
    const int index = combo->count();
    combo->addItem(QString{}, data);
    bind(combo, [combo, index, phrase = std::move(phrase)] {
        const QSignalBlocker block(combo);
        combo->setItemText(index, phrase());
    });
}
}
