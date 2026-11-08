#pragma once

#include <QApplication>
#include <QHeaderView>
#include <QHoverEvent>
#include <QItemSelectionModel>
#include <QPainter>
#include <QPaintEvent>
#include <QStyledItemDelegate>
#include <QStyleOptionViewItem>
#include <QTableView>

class FullRowTable final : public QTableView {
    class RowDelegate final : public QStyledItemDelegate {
    public:
        explicit RowDelegate(FullRowTable &table) : QStyledItemDelegate(&table), table_(table) {}

        void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override
        {
            auto rowOption = option;
            rowOption.state.setFlag(QStyle::State_HasFocus, false);
            rowOption.state.setFlag(QStyle::State_MouseOver, index.row() == table_.hoveredRow());
            QStyledItemDelegate::paint(painter, rowOption, index);
        }

    private:
        const FullRowTable &table_;
    };

public:
    FullRowTable()
    {
        setSelectionBehavior(SelectRows);
        setSelectionMode(SingleSelection);
        setEditTriggers(NoEditTriggers);
        setAlternatingRowColors(true);
        setShowGrid(false);
        viewport()->setAttribute(Qt::WA_Hover);
        setItemDelegate(new RowDelegate(*this));
    }

    QModelIndex indexAt(const QPoint &position) const override
    {
        const auto index = QTableView::indexAt(position);
        if (index.isValid() || !trailingArea().contains(position))
            return index;
        return model()->index(rowAt(position.y()), 0, rootIndex());
    }

protected:
    void paintEvent(QPaintEvent *event) override
    {
        QTableView::paintEvent(event);
        QPainter painter(viewport());
        const auto tail = trailingArea();
        if (tail.intersects(event->rect())) {
            QStyleOptionViewItem option;
            initViewItemOption(&option);
            option.viewItemPosition = QStyleOptionViewItem::End;
            const int hovered = hoveredRow();
            for (int row = rowAt(event->rect().top()); row >= 0 && row < model()->rowCount(); ++row) {
                const auto bounds = rowBounds(row);
                if (bounds.top() > event->rect().bottom())
                    break;
                option.rect = bounds.intersected(tail);
                const bool selected = selectionModel()->isRowSelected(row, rootIndex());
                if (selected || row == hovered) {
                    option.state.setFlag(QStyle::State_Selected, selected);
                    option.state.setFlag(QStyle::State_MouseOver, row == hovered);
                    style()->drawPrimitive(QStyle::PE_PanelItemViewItem, &option, &painter, this);
                } else {
                    // The item style does not paint AlternateBase beyond the final cell.
                    const bool alternate = alternatingRowColors() && row % 2;
                    const auto role = alternate ? QPalette::AlternateBase : QPalette::Base;
                    painter.fillRect(option.rect, option.palette.brush(role));
                }
            }
        }
        if (hasFocus() && currentIndex().isValid() && property("keyboardFocusVisible").toBool()) {
            painter.setPen(QApplication::palette().color(QPalette::Highlight));
            painter.drawRect(rowBounds(currentIndex().row()).adjusted(0, 0, -1, -1));
        }
    }

    QRegion visualRegionForSelection(const QItemSelection &selection) const override
    {
        QRegion region;
        for (const auto &rect : QTableView::visualRegionForSelection(selection))
            region += QRect(0, rect.y(), viewport()->width(), rect.height());
        return region;
    }

    bool viewportEvent(QEvent *event) override
    {
        if (event->type() == QEvent::HoverEnter || event->type() == QEvent::HoverMove)
            moveHover(static_cast<QHoverEvent *>(event)->position().toPoint());
        else if (event->type() == QEvent::HoverLeave || event->type() == QEvent::Leave)
            moveHover(QPoint(-1, -1));
        return QTableView::viewportEvent(event);
    }

    void scrollContentsBy(int dx, int dy) override
    {
        QTableView::scrollContentsBy(dx, dy);
        if (hoverPosition_.y() >= 0 || property("keyboardFocusVisible").toBool())
            viewport()->update();
    }

    void currentChanged(const QModelIndex &current, const QModelIndex &previous) override
    {
        QTableView::currentChanged(current, previous);
        updateRow(previous.row());
        updateRow(current.row());
    }

    bool event(QEvent *event) override
    {
        if (event->type() == QEvent::DynamicPropertyChange &&
            static_cast<QDynamicPropertyChangeEvent *>(event)->propertyName() == "keyboardFocusVisible")
            updateRow(currentIndex().row());
        return QTableView::event(event);
    }

private:
    QRect trailingArea() const
    {
        // Keep saved column widths user-resizable and treat remaining space as part of the row.
        const int dataWidth = horizontalHeader()->length() - horizontalHeader()->offset();
        const int width = viewport()->width() - dataWidth;
        if (width <= 0 || dataWidth <= 0)
            return {};
        return QRect(isRightToLeft() ? 0 : dataWidth, 0, width, viewport()->height());
    }

    int hoveredRow() const
    {
        return viewport()->rect().contains(hoverPosition_) ? rowAt(hoverPosition_.y()) : -1;
    }

    void moveHover(const QPoint &position)
    {
        const int previous = hoveredRow();
        hoverPosition_ = position;
        const int next = hoveredRow();
        if (previous != next) {
            updateRow(previous);
            updateRow(next);
        }
    }

    QRect rowBounds(int row) const
    {
        return QRect(0, rowViewportPosition(row), viewport()->width(), rowHeight(row));
    }

    void updateRow(int row)
    {
        if (row >= 0)
            viewport()->update(rowBounds(row));
    }

    QPoint hoverPosition_{-1, -1};
};
