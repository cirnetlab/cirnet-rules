#pragma once

#include "interaction_cases.h"
#include "full_row_table.h"

#include <QHeaderView>
#include <QHoverEvent>
#include <QScopeGuard>
#include <QStandardItemModel>
#include <QTableView>

namespace table_visual {

inline QColor pixelAt(const QPixmap &capture, const QPoint &point)
{
    const auto dpr = capture.devicePixelRatio();
    return capture.toImage().pixelColor(qRound(point.x() * dpr), qRound(point.y() * dpr));
}

inline QPoint cellSample(const QTableView &table, int row, int column)
{
    const auto cell = table.visualRect(table.model()->index(row, column));
    return {cell.right() - 5, cell.top() + 3};
}

inline void sendHover(QWidget &viewport, QEvent::Type type, const QPoint &point)
{
    QHoverEvent event(type, point, viewport.mapToGlobal(point), QPointF(-1, -1));
    QApplication::sendEvent(&viewport, &event);
}

inline void verifyFullRowHover(QTableView &table, const QString &captureName, bool hasTrailingSpace = false)
{
    auto &viewport = *table.viewport();
    const auto first = cellSample(table, 0, 0);
    const auto last = cellSample(table, 0, table.model()->columnCount() - 1);
    const QPoint tail(viewport.width() - 8, first.y());
    QVERIFY(viewport.rect().contains(first));
    QVERIFY(viewport.rect().contains(last));
    if (hasTrailingSpace)
        QVERIFY(tail.x() > last.x() + 10);

    const auto leave = [&] { sendHover(viewport, QEvent::HoverLeave, QPoint(-1, -1)); };
    leave();
    const auto clearHover = qScopeGuard(leave);
    const auto base = viewport.grab();
    const auto baseColor = pixelAt(base, first);
    const auto hovered = [&](const QPoint &position) {
        sendHover(viewport, QEvent::HoverMove, position);
        const auto frame = viewport.grab();
        const auto color = pixelAt(frame, first);
        QVERIFY2(color != baseColor, "Hover must visibly change the row background");
        QCOMPARE(pixelAt(frame, last), color);
        if (hasTrailingSpace)
            QCOMPARE(pixelAt(frame, tail), color);
    };
    hovered(first);
    if (hasTrailingSpace) {
        leave();
        hovered(tail);
    }
    interaction_cases::capture(table, captureName);
    leave();
    QCOMPARE(pixelAt(viewport.grab(), first), baseColor);
}

inline void verifyAlternateRowTail(QTableView &table, const QString &captureSuffix = {})
{
    auto &header = *table.horizontalHeader();
    auto &viewport = *table.viewport();
    QList<int> widths;
    const auto restore = qScopeGuard([&] {
        for (int column = 0; column < widths.size(); ++column)
            header.resizeSection(column, widths.at(column));
    });
    if (viewport.width() <= header.length() + 32) {
        for (int column = 0; column < header.count(); ++column) {
            widths.append(header.sectionSize(column));
            header.resizeSection(column, 76);
        }
    }

    QVERIFY(viewport.width() > header.length() + 32);
    sendHover(viewport, QEvent::HoverLeave, QPoint(-1, -1));
    const auto base = viewport.grab();
    const auto even = cellSample(table, 0, 0);
    const auto odd = cellSample(table, 1, 0);
    const int tail = viewport.width() - 8;
    QVERIFY2(pixelAt(base, even) != pixelAt(base, odd), "Both alternate bands must be visible");
    QCOMPARE(pixelAt(base, QPoint(tail, even.y())), pixelAt(base, even));
    QCOMPARE(pixelAt(base, QPoint(tail, odd.y())), pixelAt(base, odd));
    interaction_cases::capture(table, QStringLiteral("repositories-full-width-stripes") + captureSuffix);
    verifyFullRowHover(table, QStringLiteral("repositories-full-row-hover") + captureSuffix, true);
}

inline void liveThemeAndKeyboardFocus()
{
    ThemeManager theme;
    theme.apply("light", "comfortable", 100, {{"accent", "cobalt"}});
    QStandardItemModel model(3, 2);
    model.setHorizontalHeaderLabels({"Name", "State"});
    model.setData(model.index(0, 0), "Fixture");
    FullRowTable table;
    table.setModel(&model);
    table.verticalHeader()->hide();
    table.setColumnWidth(0, 105);
    table.setColumnWidth(1, 95);
    table.resize(375, 180);
    interaction_cases::pinToPrimary(table);
    table.show();
    table.activateWindow();
    auto &viewport = *table.viewport();
    const auto first = cellSample(table, 0, 0);
    const auto last = cellSample(table, 0, 1);
    const QPoint tail(viewport.width() - 8, first.y());
    QVERIFY(tail.x() > last.x() + 10);
    QTest::mouseClick(&viewport, Qt::LeftButton, Qt::NoModifier, first);
    QTRY_VERIFY(table.hasFocus());
    QVERIFY(table.selectionModel()->isRowSelected(0));
    const auto selected = viewport.grab();
    const auto cobalt = pixelAt(selected, first);
    QCOMPARE(pixelAt(selected, last), cobalt);
    QCOMPARE(pixelAt(selected, tail), cobalt);

    QTest::keyClick(&table, Qt::Key_Control);
    QTest::keyClick(&table, Qt::Key_C, Qt::ControlModifier);
    QVERIFY(!table.property("keyboardFocusVisible").toBool());
    QCOMPARE(viewport.grab().toImage(), selected.toImage());

    theme.apply("light", "comfortable", 100, {{"accent", "purple"}});
    const auto purpleRow = viewport.grab();
    const auto purple = pixelAt(purpleRow, first);
    QVERIFY2(purple != cobalt, "Selection should use the active accent instead of a neutral fill");
    QCOMPARE(pixelAt(purpleRow, last), purple);
    QCOMPARE(pixelAt(purpleRow, tail), purple);
    auto &header = *table.horizontalHeader();
    const auto headerImage = header.grab();
    for (const int y : {4, header.height() - 1})
        QCOMPARE(pixelAt(headerImage, {95, y}), pixelAt(headerImage, {header.width() - 8, y}));

    QTest::keyClick(&table, Qt::Key_Down);
    QCOMPARE(table.currentIndex().row(), 1);
    QVERIFY(table.property("keyboardFocusVisible").toBool());
    const auto keyboardRow = viewport.grab();
    const auto next = cellSample(table, 1, 0);
    QCOMPARE(pixelAt(keyboardRow, {3, next.y()}), table.palette().color(QPalette::Link));
    QCOMPARE(pixelAt(keyboardRow, {tail.x(), next.y() - 3}), pixelAt(keyboardRow, next));
}

} // namespace table_visual
