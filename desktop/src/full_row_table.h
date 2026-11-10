#pragma once

#include "theme.h"

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
            auto cell = option;
            initStyleOption(&cell, index);
            painter->fillRect(cell.rect, table_.rowBackground(index.row(), cell.palette));
            cell.state &= ~(QStyle::State_HasFocus | QStyle::State_MouseOver | QStyle::State_Selected);
            cell.features.setFlag(QStyleOptionViewItem::Alternate, false);
            cell.backgroundBrush = QBrush{};
            table_.style()->drawControl(QStyle::CE_ItemViewItem, &cell, painter, &table_);
        }

    private:
        const FullRowTable &table_;
    };

public:
    FullRowTable()
    {
        setProperty("dataTable", true);
        setSelectionBehavior(SelectRows);
        setSelectionMode(SingleSelection);
        setEditTriggers(NoEditTriggers);
        setAlternatingRowColors(true);
        setShowGrid(false);
        horizontalHeader()->setTextElideMode(Qt::ElideRight);
        viewport()->setAttribute(Qt::WA_Hover);
        setItemDelegate(new RowDelegate(*this));
    }

    QModelIndex indexAt(const QPoint &position) const override
    {
        const auto index = QTableView::indexAt(position);
        if (index.isValid() || !model() || !trailingArea().contains(position))
            return index;
        return model()->index(rowAt(position.y()), 0, rootIndex());
    }

protected:
    void paintEvent(QPaintEvent *event) override
    {
        QTableView::paintEvent(event);
        if (!model() || !selectionModel())
            return;
        QPainter painter(viewport());
        const auto tail = trailingArea();
        const bool paintTail = tail.intersects(event->rect());
        const auto colors = palette();
        const auto divider = colors.brush(QPalette::Dark);
        const auto accent = colors.brush(QPalette::Link);
        const int focusRow = hasFocus() && property("keyboardFocusVisible").toBool()
            ? currentIndex().row() : -1;
        const int rowCount = model()->rowCount(rootIndex());
        for (int row = rowAt(event->rect().top()); row >= 0 && row < rowCount; ++row) {
            const auto bounds = rowBounds(row);
            if (bounds.top() > event->rect().bottom())
                break;
            if (bounds.isEmpty())
                continue;
            const bool selected = selectionModel()->isRowSelected(row, rootIndex());
            if (paintTail)
                painter.fillRect(bounds.intersected(tail), rowBackground(row, colors));
            painter.fillRect(bounds.left(), bounds.bottom(), bounds.width(), 1, divider);
            if (selected || row == focusRow) {
                const int width = row == focusRow ? 4 : 2;
                const int edge = isRightToLeft() ? bounds.right() + 1 - width : bounds.left();
                painter.fillRect(edge, bounds.top(), width, bounds.height() - 1, accent);
            }
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
    QColor rowBackground(int row, const QPalette &colors) const
    {
        if (selectionModel()->isRowSelected(row, rootIndex()))
            return uiTone(UiTone::TableSelection);
        if (row == hoveredRow())
            return uiTone(UiTone::TableHover);
        return colors.color(alternatingRowColors() && row % 2 ? QPalette::AlternateBase : QPalette::Base);
    }

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
