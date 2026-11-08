#pragma once

#include "main_window.h"
#include "interaction_cases.h"

#include <QAccessible>
#include <QCheckBox>
#include <QComboBox>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMessageBox>
#include <QPushButton>
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
    QCOMPARE(navigation->count(), 8);
    QStringList sections;
    for (int i = 0; i < navigation->count(); ++i)
        sections.append(navigation->item(i)->data(Qt::UserRole).toString());
    QVERIFY(sections.contains("configuration") && sections.contains("storage") && sections.contains("archived"));

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

    auto *accent = window.findChild<QComboBox *>("accentChoice");
    auto *reset = window.findChild<QPushButton *>("resetAppearance");
    QVERIFY(accent && reset);
    const auto defaultAccent = window.palette().color(QPalette::Highlight);
    accent->setCurrentIndex(accent->findData("cobalt"));
    QTRY_VERIFY(window.palette().color(QPalette::Highlight) != defaultAccent);
    reset->click();
    QTRY_COMPARE(window.palette().color(QPalette::Highlight), defaultAccent);
    QTest::keyClick(motion, Qt::Key_Space);

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
    navigation->setCurrentRow(sections.indexOf("configuration"));
    auto *adapter = window.findChild<QComboBox *>("configurationAdapter");
    auto *capability = window.findChild<QLabel *>("configurationCapability");
    QVERIFY(adapter && capability);
    if (adapter->count() > 0)
        QVERIFY(capability->text().contains("HANDOFF_ONLY"));
    QTest::keyClick(navigation, Qt::Key_Escape);
    QVERIFY(!navigation->isVisible());
}

template <class Request>
void verifyRuntimeSettingsData(MainWindow &window, const QString &bindingId, Request request)
{
    const QString title = QStringLiteral("Archived Settings fixture");
    const auto created = request("conversation.create", {{"title", title}, {"bindings", QJsonArray{bindingId}},
        {"profile", "handoff"}, {"key", "archived-settings-fixture"}});
    const auto conversation = created.value("conversation").isObject()
        ? created.value("conversation").toObject() : created;
    const auto id = conversation.value("id");
    QVERIFY(!id.toString().isEmpty());
    QVERIFY(!request("conversation.archive", {{"id", id}, {"revision", conversation.value("revision")}}).isEmpty());

    auto *settings = window.findChild<QToolButton *>("navSettings");
    auto *navigation = window.findChild<QListWidget *>("settingsNavigation");
    QVERIFY(settings && navigation);
    if (!navigation->isVisible())
        settings->click();
    const auto selectPage = [navigation](const QString &key) {
        for (int row = 0; row < navigation->count(); ++row) {
            if (navigation->item(row)->data(Qt::UserRole).toString() == key) {
                navigation->setCurrentRow(row);
                return;
            }
        }
        QFAIL("The requested Settings category is missing.");
    };
    selectPage("archived");
    auto *search = window.findChild<QLineEdit *>("archivedChatSearch");
    auto *chats = window.findChild<QListWidget *>("archivedChats");
    auto *unarchive = window.findChild<QPushButton *>("unarchiveChat");
    QVERIFY(search && chats && unarchive);
    search->setText(title);
    QTRY_VERIFY(chats->count() == 1 && chats->item(0)->text().contains(title));
    interaction_cases::capture(window, "archived-chats-real-runtime");
    chats->setCurrentRow(0);
    QTRY_VERIFY(unarchive->isEnabled());
    unarchive->click();
    QTRY_COMPARE(chats->count(), 0);
    const auto active = request("conversation.list", {{"state", "ACTIVE"}, {"query", title}}).value("items").toArray();
    QCOMPARE(active.size(), 1);
    QCOMPARE(active[0].toObject().value("id"), id);
    QVERIFY(!request("conversation.archive", {{"id", id}, {"revision", active[0].toObject().value("revision")}}).isEmpty());
    auto *refresh = window.findChild<QPushButton *>("archivedRefresh");
    auto *remove = window.findChild<QPushButton *>("deleteArchivedChat");
    QVERIFY(refresh && remove);
    refresh->click();
    QTRY_COMPARE(chats->count(), 1);
    chats->setCurrentRow(0);
    QTRY_VERIFY(remove->isEnabled());
    remove->click();
    QTRY_VERIFY(window.findChild<QMessageBox *>("deleteDataConfirmation"));
    auto *confirmation = window.findChild<QMessageBox *>("deleteDataConfirmation");
    QVERIFY(confirmation->text().contains(title));
    confirmation->button(QMessageBox::Yes)->click();
    QTRY_COMPARE(chats->count(), 0);
    QCOMPARE(request("conversation.list", {{"state", "ARCHIVED"}, {"query", title}}).value("total").toInt(-1), 0);

    selectPage("storage");
    auto *scan = window.findChild<QPushButton *>("scanStorage");
    auto *accounting = window.findChild<QLabel *>("storageAccounting");
    auto *status = window.findChild<QLabel *>("storageOperationStatus");
    auto *backup = window.findChild<QPushButton *>("createStorageBackup");
    auto *verify = window.findChild<QPushButton *>("verifyStorageBackup");
    auto *backupSelector = window.findChild<QComboBox *>("storageBackups");
    QVERIFY(scan && accounting && status && backup && verify && backupSelector);
    scan->click();
    QTRY_VERIFY(accounting->text().contains("Reusable database pages"));
    interaction_cases::capture(window, "storage-real-runtime");
    backup->click();
    QTRY_VERIFY(backup->isEnabled() && status->text().contains("completed"));
    QVERIFY(!backupSelector->currentData().toString().isEmpty());
    verify->click();
    QTRY_VERIFY(verify->isEnabled() && status->text().contains("completed"));

    selectPage("configuration");
    auto *capability = window.findChild<QLabel *>("configurationCapability");
    QVERIFY(capability && capability->text().contains("HANDOFF_ONLY"));
    interaction_cases::capture(window, "configuration-real-runtime");
}
