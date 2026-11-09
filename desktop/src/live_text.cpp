#include "live_text.h"

#include <QCoreApplication>
#include <QLabel>
#include <QLocale>
#include <QTranslator>

namespace live {
Language::Language(QObject *parent) : QObject(parent) {}
Language::~Language() = default;

bool Language::set(QString locale)
{
    locale = locale.trimmed().toLower().replace(u'-', u'_');
    if (locale == "vi_vn")
        locale = "vi";
    else if (locale == "zh" || locale == "zh_cn")
        locale = "zh_CN";
    else if (locale != "vi")
        locale = "en";
    if (locale_ == locale)
        return true;

    std::unique_ptr<QTranslator> next;
    if (locale != "en") {
        next = std::make_unique<QTranslator>();
        if (!next->load(QStringLiteral(":/i18n/openrules_%1.qm").arg(locale)))
            return false;
    }
    if (translator_)
        QCoreApplication::removeTranslator(translator_.get());
    translator_ = std::move(next);
    if (translator_)
        QCoreApplication::installTranslator(translator_.get());
    QLocale::setDefault(QLocale(locale));
    locale_ = locale;
    emit changed();
    return true;
}

Language &language()
{
    static auto *instance = new Language(QCoreApplication::instance());
    return *instance;
}

class MessageBinding final : public QObject {
    Q_OBJECT
public:
    explicit MessageBinding(QLabel *label) : QObject(label), label_(label)
    {
        connect(&language(), &Language::changed, this, [this] { label_->setText(phrase()); });
    }
    Phrase phrase;

private:
    QLabel *label_;
};

void message(QLabel *label, Phrase phrase)
{
    auto *binding = label->findChild<MessageBinding *>(Qt::FindDirectChildrenOnly);
    if (!binding)
        binding = new MessageBinding(label);
    binding->phrase = std::move(phrase);
    label->setText(binding->phrase());
}
}

#include "live_text.moc"
