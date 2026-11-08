#pragma once

#include "main_window.h"

#include <QAccessible>
#include <QCheckBox>
#include <QComboBox>
#include <QLineEdit>
#include <QListWidget>
#include <QRadioButton>
#include <QStackedWidget>
#include <QTest>
#include <QToolButton>

inline void verifySettingsPolish(MainWindow &window)
{
    auto *settings = window.findChild<QToolButton *>("navSettings");
    QVERIFY(settings);
    QTest::mouseClick(settings, Qt::LeftButton);

    auto *navigation = window.findChild<QListWidget *>("settingsNavigation");
    auto *search = window.findChild<QLineEdit *>("settingsSearch");
    auto *pages = window.findChild<QStackedWidget *>("settingsPages");
    QVERIFY2(navigation && search && pages, "Settings must have a left category list and local search");
    QVERIFY(!window.findChild<QComboBox *>("settingsSection"));
    QVERIFY(navigation->mapTo(&window, QPoint(navigation->width(), 0)).x() <=
            pages->mapTo(&window, QPoint{}).x());
    QCOMPARE(navigation->count(), 5);

    QTest::keyClicks(search, "motion");
    QVERIFY(!navigation->item(0)->isHidden());
    for (int i = 1; i < navigation->count(); ++i)
        QVERIFY(navigation->item(i)->isHidden());
    QTest::keyClick(search, Qt::Key_Return);
    auto *motion = window.findChild<QCheckBox *>("reducedMotion");
    QVERIFY(motion && motion->hasFocus());
    QVERIFY(!motion->isChecked());
    QTest::keyClick(motion, Qt::Key_Space);
    QVERIFY(motion->isChecked());

    search->clear();
    auto *light = window.findChild<QRadioButton *>("themeLight");
    auto *dark = window.findChild<QRadioButton *>("themeDark");
    auto *system = window.findChild<QRadioButton *>("themeSystem");
    QVERIFY(light && dark && system);
    const auto *radioInterface = QAccessible::queryAccessibleInterface(light);
    QVERIFY(radioInterface && radioInterface->role() == QAccessible::RadioButton);
    QCOMPARE(light->height(), dark->height());
    QCOMPARE(light->height(), system->height());
    QVERIFY(qAbs(light->width() - dark->width()) <= 1);
    QVERIFY(qAbs(light->width() - system->width()) <= 1);
    QTest::mouseClick(light, Qt::LeftButton);
    QVERIFY(light->isChecked());
    QVERIFY(!dark->isChecked() && !system->isChecked());
    QVERIFY(window.palette().color(QPalette::Window).lightness() > 128);
    QTest::keyClick(dark, Qt::Key_Space);
    QVERIFY(dark->isChecked());
    QVERIFY(!light->isChecked());
    QVERIFY(window.palette().color(QPalette::Window).lightness() < 128);

    const QSize originalSize = window.size();
    window.resize(window.minimumWidth(), 400);
    QTRY_VERIFY(system->mapTo(&window, QPoint{}).x() >= pages->mapTo(&window, QPoint{}).x());
    QTRY_VERIFY(dark->mapTo(&window, QPoint(dark->width(), 0)).x() <= window.width());
    window.resize(originalSize);

    auto *density = window.findChild<QComboBox *>("densityChoice");
    QVERIFY(density);
    density->showPopup();
    QTRY_VERIFY(density->view()->isVisible());
    QTest::keyClick(density->view(), Qt::Key_Escape);
    QTRY_VERIFY(!density->view()->isVisible());
    QVERIFY(navigation->isVisible());

    navigation->setFocus();
    QTest::keyClick(navigation, Qt::Key_Down);
    QCOMPARE(navigation->currentItem()->data(Qt::UserRole).toString(), QString("repositories"));
    QCOMPARE(pages->currentWidget()->objectName(), QString("repositoriesPage"));
    QTest::keyClick(navigation, Qt::Key_Escape);
    QVERIFY(!navigation->isVisible());
}
