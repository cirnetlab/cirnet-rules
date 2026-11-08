#pragma once

#include <QPlainTextEdit>
#include <QTextDocument>
#include <QtMath>

class CompactTextEdit final : public QPlainTextEdit {
public:
    explicit CompactTextEdit(int lines, QWidget *parent = nullptr) : QPlainTextEdit(parent), lines_(lines)
    {
        setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    }

    QSize sizeHint() const override { return lineSize(lines_); }
    QSize minimumSizeHint() const override { return lineSize(2); }

private:
    QSize lineSize(int lines) const
    {
        const auto padding = 2 * (frameWidth() + qCeil(document()->documentMargin())) + 8;
        return QSize(QPlainTextEdit::minimumSizeHint().width(), lines * fontMetrics().lineSpacing() + padding);
    }

    int lines_;
};
