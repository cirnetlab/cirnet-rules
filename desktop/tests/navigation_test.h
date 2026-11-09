#pragma once

#include "main_window.h"

#include <QDialog>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QStackedWidget>
#include <QTest>
#include <QToolButton>
#include <algorithm>

inline void captureNavigation(MainWindow &window, const QString &name)
{
    const auto directory = qEnvironmentVariable("CIRNETRULES_UI_CAPTURE");
    if (!directory.isEmpty())
        QVERIFY(window.grab().save(directory + "/" + name + ".png"));
}

inline void verifyExclusiveNavigation(MainWindow &window)
{
    auto *history = window.findChild<QListWidget *>("conversationList");
    auto *automation = window.findChild<QPushButton *>("navAutomations");
    auto *composer = window.findChild<QPlainTextEdit *>("conversationComposer");
    auto *settings = window.findChild<QToolButton *>("navSettings");
    auto *back = window.findChild<QPushButton *>("settingsBack");
    QVERIFY(history && automation && composer && settings && back);
    QTRY_COMPARE(history->count(), 2);
    const auto open = [history](int row) {
        QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                          history->visualItemRect(history->item(row)).center());
    };
    open(0);
    QTRY_VERIFY(composer->isVisible() && !composer->isReadOnly());
    composer->setPlainText("Preserve my navigation draft.");
    QCOMPARE(history->selectedItems(), QList<QListWidgetItem *>{history->item(0)});
    QVERIFY(!automation->isChecked());
    captureNavigation(window, "nav-chat-active");

    QTest::mouseClick(automation, Qt::LeftButton);
    captureNavigation(window, "nav-automations-after-chat");
    QVERIFY(automation->isChecked());
    QVERIFY(history->selectedItems().isEmpty());
    QTest::keyClick(history, Qt::Key_End);
    QCOMPARE(history->currentItem(), history->item(1));
    QVERIFY(history->selectedItems().isEmpty());

    QTest::mouseClick(settings, Qt::LeftButton);
    QVERIFY(settings->isChecked());
    QVERIFY(!automation->isChecked());
    QVERIFY(history->selectedItems().isEmpty());
    QVERIFY(!history->isVisible());
    QTest::mouseClick(back, Qt::LeftButton);
    QVERIFY(automation->isChecked());
    QVERIFY(history->isVisible());

    open(1);
    QTRY_VERIFY(composer->isVisible() && !composer->isReadOnly());
    QCOMPARE(history->selectedItems(), QList<QListWidgetItem *>{history->item(1)});
    QVERIFY(!automation->isChecked());
    open(0);
    QCOMPARE(composer->toPlainText(), QString("Preserve my navigation draft."));
}

inline void verifyNewChatCancelKeepsRoute(MainWindow &window)
{
    auto *automation = window.findChild<QPushButton *>("navAutomations");
    auto *chat = window.findChild<QPushButton *>("navNewChat");
    QVERIFY(automation && chat);
    QTRY_VERIFY(window.findChild<QPushButton *>("addAutomation")->isEnabled());
    QVERIFY(automation->isChecked());
    QTest::mouseClick(chat, Qt::LeftButton);
    QTRY_VERIFY(window.findChild<QDialog *>("newConversationDialog"));
    QTest::keyClick(window.findChild<QDialog *>("newConversationDialog"), Qt::Key_Escape);
    QTRY_VERIFY(!window.findChild<QDialog *>("newConversationDialog"));
    QVERIFY(automation->isChecked());
    QVERIFY(window.findChild<QWidget *>("automationsPage")->isVisible());
    QVERIFY(!chat->isChecked());
}

inline void verifyCollapsedSidebarGrouping(MainWindow &window)
{
    auto *rail = window.findChild<QWidget *>("sidebar");
    auto *logo = window.findChild<QLabel *>("brandMark");
    auto *toggle = window.findChild<QToolButton *>("toggleSidebar");
    auto *chat = window.findChild<QPushButton *>("navNewChat");
    auto *automation = window.findChild<QPushButton *>("navAutomations");
    auto *footer = window.findChild<QWidget *>("sidebarFooter");
    QVERIFY(rail && logo && toggle && chat && automation && footer);
    auto *local = footer->findChild<QToolButton *>("localStatus");
    auto *settings = footer->findChild<QToolButton *>("navSettings");
    auto *back = window.findChild<QPushButton *>("settingsBack");
    QVERIFY(local && settings && back);
    const auto y = [rail](const QWidget *widget) { return widget->mapTo(rail, QPoint(0, 0)).y(); };
    const auto footerFits = [&] {
        const auto contains = [](const QWidget *parent, const QWidget *child) {
            return parent->rect().contains(QRect(child->mapTo(parent, QPoint(0, 0)), child->size()));
        };
        return contains(&window, rail) && rail->mapTo(&window, QPoint(0, rail->height())).y() == window.height()
            && contains(rail, footer) && rail->height() - y(footer) - footer->height() <= 2 && local->isVisible()
            && settings->isVisible() && contains(footer, local) && contains(footer, settings);
    };
    QTRY_COMPARE(window.findChildren<QLabel *>("automationCardTitle").size(), 3);
    QTest::mouseClick(toggle, Qt::LeftButton);
    QTRY_VERIFY(!window.findChild<QWidget *>("conversationHistory")->isVisible());
    window.resize(1050, 460);
    QTRY_VERIFY(footerFits());
    const QList<int> compactPositions{y(logo), y(toggle), y(chat), y(automation)};
    captureNavigation(window, "nav-rail-collapsed-short");
    window.resize(1050, 820);
    QTRY_VERIFY(footerFits());
    const QList<int> tallPositions{y(logo), y(toggle), y(chat), y(automation)};
    captureNavigation(window, "nav-rail-collapsed-tall");
    QVERIFY2(tallPositions.first() <= 28 && tallPositions.last() + automation->height() <= 212,
             qPrintable(QString("Collapsed controls begin at %1 and end at %2")
                            .arg(tallPositions.first()).arg(tallPositions.last() + automation->height())));
    QCOMPARE(tallPositions, compactPositions);
    QVERIFY(toggle->height() >= 36 && toggle->width() >= 36);
    chat->setFocus(Qt::TabFocusReason);
    QTest::keyClick(chat, Qt::Key_Space);
    QTRY_VERIFY(window.findChild<QDialog *>("newConversationDialog"));
    QTest::keyClick(window.findChild<QDialog *>("newConversationDialog"), Qt::Key_Escape);
    QTRY_VERIFY(!window.findChild<QDialog *>("newConversationDialog"));
    QTest::keyClick(automation, Qt::Key_Space);
    QVERIFY(automation->isChecked());
    QTest::keyClick(settings, Qt::Key_Space);
    QVERIFY(settings->isChecked());
    QVERIFY(!automation->isChecked());
    QTest::keyClick(back, Qt::Key_Space);
    QVERIFY(automation->isChecked());
    QVERIFY(!window.findChild<QWidget *>("conversationHistory")->isVisible());
}

template <class Request>
void verifyLateConversationReplies(MainWindow &window, Request invoke)
{
    auto *history = window.findChild<QListWidget *>("conversationList");
    auto *search = window.findChild<QLineEdit *>("conversationSearch");
    auto *automation = window.findChild<QPushButton *>("navAutomations");
    auto *chatPage = window.findChild<QWidget *>("conversationsPage");
    QVERIFY(chatPage);
    auto *title = chatPage->findChild<QLabel *>("heading");
    QVERIFY(history && search && automation && title);
    QTRY_COMPARE(history->count(), 2);
    const auto open = [history](int row) {
        QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                          history->visualItemRect(history->item(row)).center());
    };
    open(0); // Held by the fixture.
    open(1);
    QTRY_COMPARE(title->text(), QString("Other chat"));
    search->setText("delayed refresh");
    const auto refreshSent = [&] {
        const auto methods = invoke("test.methods").value("items").toArray();
        return std::count(methods.begin(), methods.end(), QJsonValue("conversation.list")) >= 2;
    };
    QTRY_VERIFY(refreshSent());
    QTest::mouseClick(automation, Qt::LeftButton);
    invoke("test.releaseNavigation");
    QCOMPARE(title->text(), QString("Other chat"));
    QVERIFY(automation->isChecked());
    QVERIFY(!chatPage->isVisible());
    QVERIFY(history->selectedItems().isEmpty());
    open(1);
    QVERIFY(chatPage->isVisible());
    QCOMPARE(history->selectedItems(), QList<QListWidgetItem *>{history->item(1)});
    QCOMPARE(title->text(), QString("Other chat"));
}
