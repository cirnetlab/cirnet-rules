#pragma once

#include <QAbstractTextDocumentLayout>
#include <QColor>
#include <QPaintEvent>
#include <QPainter>
#include <QPlainTextEdit>
#include <QResizeEvent>
#include <QSyntaxHighlighter>
#include <QTextBlock>
#include <QTextBrowser>

class GovernanceEditor final : public QPlainTextEdit {
public:
    explicit GovernanceEditor(QWidget *parent = nullptr)
        : QPlainTextEdit(parent), gutter_(new Gutter(*this)),
          highlighter_(new Highlighter(document(), *this))
    {
        setProperty("codeText", true);
        setTabChangesFocus(true);
        connect(this, &QPlainTextEdit::blockCountChanged, this, [this] { placeGutter(); });
        connect(this, &QPlainTextEdit::updateRequest, gutter_, [this](const QRect &rect, int dy) {
            if (dy)
                gutter_->scroll(0, dy);
            else
                gutter_->update(0, rect.y(), gutter_->width(), rect.height());
        });
        placeGutter();
    }

protected:
    void resizeEvent(QResizeEvent *event) override
    {
        QPlainTextEdit::resizeEvent(event);
        placeGutter();
    }

    void changeEvent(QEvent *event) override
    {
        QPlainTextEdit::changeEvent(event);
        if (event->type() == QEvent::FontChange)
            placeGutter();
        if (event->type() == QEvent::PaletteChange) {
            highlighter_->rehighlight();
            gutter_->update();
        }
    }

private:
    class Gutter final : public QWidget {
    public:
        explicit Gutter(GovernanceEditor &editor) : QWidget(&editor), editor_(editor) {}
    protected:
        void paintEvent(QPaintEvent *event) override
        {
            QPainter painter(this);
            painter.fillRect(event->rect(), editor_.palette().color(QPalette::AlternateBase));
            painter.setPen(editor_.palette().color(QPalette::PlaceholderText));
            auto block = editor_.firstVisibleBlock();
            auto top = editor_.blockBoundingGeometry(block).translated(editor_.contentOffset()).top();
            while (block.isValid() && top <= event->rect().bottom()) {
                const auto height = editor_.blockBoundingRect(block).height();
                if (block.isVisible() && top + height >= event->rect().top()) {
                    painter.drawText(0, qRound(top), width() - 5, editor_.fontMetrics().height(),
                                     Qt::AlignRight, QString::number(block.blockNumber() + 1));
                }
                block = block.next();
                top += height;
            }
        }
    private:
        GovernanceEditor &editor_;
    };

    class Highlighter final : public QSyntaxHighlighter {
    public:
        Highlighter(QTextDocument *document, const GovernanceEditor &editor)
            : QSyntaxHighlighter(document), editor_(editor) {}
    protected:
        void highlightBlock(const QString &text) override
        {
            QTextCharFormat format;
            const auto trimmed = text.trimmed();
            if (trimmed.startsWith(u'#') || trimmed == QStringLiteral("---")) {
                format.setForeground(editor_.palette().color(QPalette::Link));
                format.setFontWeight(QFont::DemiBold);
                setFormat(0, text.size(), format);
                return;
            }
            const auto delimiter = text.indexOf(u':');
            if (delimiter > 0 && delimiter < 64 && !text.left(delimiter).contains(u'/')) {
                format.setFontWeight(QFont::DemiBold);
                setFormat(0, delimiter, format);
            }
        }
    private:
        const GovernanceEditor &editor_;
    };

    void placeGutter()
    {
        const auto width = 10 + fontMetrics().horizontalAdvance(u'9') * qMax(2, int(QString::number(blockCount()).size()));
        const auto rect = contentsRect();
        if (viewportMargins().left() != width)
            setViewportMargins(width, 0, 0, 0);
        gutter_->setGeometry(rect.left(), rect.top(), width, rect.height());
    }

    Gutter *gutter_;
    Highlighter *highlighter_;
};

class GovernancePreview final : public QTextBrowser {
    class LinkHighlighter final : public QSyntaxHighlighter {
    public:
        explicit LinkHighlighter(GovernancePreview &preview)
            : QSyntaxHighlighter(preview.document()), preview_(preview) {}

    protected:
        void highlightBlock(const QString &) override
        {
            const auto block = currentBlock();
            const auto color = preview_.palette().color(QPalette::Link);
            // Markdown stores the import-time link color. Override it only for painting.
            for (auto it = block.begin(); !it.atEnd(); ++it) {
                const auto fragment = it.fragment();
                if (fragment.charFormat().isAnchor())
                    setFormat(fragment.position() - block.position(), fragment.length(), color);
            }
        }

    private:
        const GovernancePreview &preview_;
    };

public:
    explicit GovernancePreview(QWidget *parent = nullptr)
        : QTextBrowser(parent), links_(new LinkHighlighter(*this))
    {
        setOpenLinks(false);
        setOpenExternalLinks(false);
    }

    void showSource(const QString &source, bool markdown)
    {
        if (markdown)
            document()->setMarkdown(source, QTextDocument::MarkdownFeatures{QTextDocument::MarkdownDialectCommonMark, QTextDocument::MarkdownNoHTML});
        else
            setPlainText(source);
    }

protected:
    void changeEvent(QEvent *event) override
    {
        QTextBrowser::changeEvent(event);
        if (event->type() == QEvent::PaletteChange)
            links_->rehighlight();
    }

    QVariant loadResource(int, const QUrl &) override { return {}; }

private:
    LinkHighlighter *links_;
};
