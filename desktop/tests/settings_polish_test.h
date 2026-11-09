#pragma once

#include "main_window.h"
#include "interaction_cases.h"
#include "settings_panels.h"

#include <QAccessible>
#include <QAction>
#include <QCheckBox>
#include <QComboBox>
#include <QDialog>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMessageBox>
#include <QPushButton>
#include <QRadioButton>
#include <QScrollArea>
#include <QSpinBox>
#include <QStackedWidget>
#include <QTest>
#include <QToolButton>

#include <memory>

inline void verifyArchivedCardQueries()
{
    QList<QJsonObject> queries;
    QList<RuntimeClient::Reply> replies;
    QString deletedId;
    std::unique_ptr<QWidget> page(makeArchivedChats(
        [&queries, &replies](const QString &method, QJsonObject params, RuntimeClient::Reply reply) {
            QCOMPARE(method, QString("conversation.list"));
            queries.append(std::move(params));
            replies.append(std::move(reply));
        }, [] { return QJsonArray{}; }, [] {},
        [&deletedId](const QString &id, const std::function<void(bool)> &finished) {
            deletedId = id;
            finished(false);
        }));
    page->resize(920, 600);
    page->show();
    QTRY_COMPARE(queries.size(), 1);
    auto *search = page->findChild<QLineEdit *>("archivedChatSearch");
    QVERIFY(search);
    search->setText("current");
    const auto complete = [&replies](qsizetype index, const QString &id, const QString &title) {
        auto reply = std::move(replies[index]);
        reply({{"items", QJsonArray{QJsonObject{{"id", id}, {"title", title}, {"revision", 1}}}},
               {"total", 1}}, {});
    };
    complete(0, "10", "Superseded result");
    QTRY_COMPARE(queries.size(), 2);
    QCOMPARE(queries[1].value("query").toString(), QString("current"));
    QVERIFY(!page->findChild<QWidget *>("settingsCard"));
    complete(1, "11", "Current result");
    auto *card = page->findChild<QWidget *>("settingsCard");
    QVERIFY(card);
    QCOMPARE(card->property("conversationId").toString(), QString("11"));
    auto *erase = card->findChild<QToolButton *>("deleteArchivedChat");
    QVERIFY(erase && erase->isEnabled());
    erase->click();
    QCOMPARE(deletedId, QString("11"));
    QVERIFY(erase->isEnabled());
}

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
    for (int i = 0; i < navigation->count(); ++i) {
        QVERIFY(!navigation->item(i)->icon().isNull());
        sections.append(navigation->item(i)->data(Qt::UserRole).toString());
    }
    const auto searchActions = search->findChildren<QAction *>("focusSearchField");
    QCOMPARE(searchActions.size(), 1);
    QVERIFY(!searchActions.first()->icon().isNull());
    QVERIFY(sections.contains("configuration") && sections.contains("storage") && sections.contains("archived"));

    auto *advanced = window.findChild<QToolButton *>("appearanceAdvancedToggle");
    auto *motion = window.findChild<QCheckBox *>("reducedMotion");
    auto *reset = window.findChild<QPushButton *>("resetAppearanceDefault");
    QVERIFY(advanced && motion && reset);
    QVERIFY(advanced->isVisible() && !advanced->isChecked());
    QVERIFY(!motion->isVisible() && !reset->isVisible());
    QTest::keyClick(advanced, Qt::Key_Space);
    QVERIFY(advanced->isChecked() && motion->isVisible());
    QTest::keyClick(advanced, Qt::Key_Space);
    QVERIFY(!advanced->isChecked() && !motion->isVisible());

    QTest::keyClicks(search, "motion");
    QVERIFY(!navigation->item(0)->isHidden());
    for (int i = 1; i < navigation->count(); ++i)
        QVERIFY(navigation->item(i)->isHidden());
    QTest::keyClick(search, Qt::Key_Return);
    QVERIFY(advanced->isChecked() && motion->hasFocus());
    QVERIFY(!motion->isChecked());
    QTest::keyClick(motion, Qt::Key_Space);
    QVERIFY(motion->isChecked());
    QTRY_VERIFY(reset->isVisible());
    QTRY_VERIFY(motion->visibleRegion().contains(motion->rect().center()));

    search->setText("Code text size");
    QTest::keyClick(search, Qt::Key_Return);
    auto *code = window.findChild<QSpinBox *>("codeScaleChoice");
    QVERIFY(code && code->hasFocus() && !navigation->item(0)->isHidden());
    QCOMPARE(code->suffix(), QStringLiteral(" px"));
    search->clear();
    QTest::keyClick(advanced, Qt::Key_Space);
    QVERIFY(!advanced->isChecked() && !motion->isVisible());
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
    auto *background = window.findChild<QLineEdit *>("appearanceBackground");
    QVERIFY(background);
    QCOMPARE(background->text(), window.palette().color(QPalette::Window).name().toUpper());
    QTest::keyClick(dark, Qt::Key_Space);
    QVERIFY(dark->isChecked());
    QVERIFY(!light->isChecked());
    QVERIFY(window.palette().color(QPalette::Window).lightness() < 128);

    auto *accent = window.findChild<QComboBox *>("accentChoice");
    QVERIFY(accent);
    QVERIFY(accent->count() >= 9);
    QVERIFY(!accent->itemIcon(accent->findData("green")).isNull());
    if (!qEnvironmentVariableIsEmpty("CGACP_UI_CAPTURE")) {
        accent->showPopup();
        QTRY_VERIFY(accent->view()->isVisible());
        interaction_cases::capture(*accent->view()->window(), "settings-accent-palette");
        QTest::keyClick(accent->view(), Qt::Key_Escape);
        QTRY_VERIFY(!accent->view()->isVisible());
        auto *fontMenu = window.findChild<QComboBox *>("uiFontChoice");
        QVERIFY(fontMenu);
        fontMenu->showPopup();
        QTRY_VERIFY(fontMenu->view()->isVisible());
        interaction_cases::capture(*fontMenu->view()->window(), "settings-font-menu");
        QTest::keyClick(fontMenu->view(), Qt::Key_Escape);
        QTRY_VERIFY(!fontMenu->view()->isVisible());
    }
    const auto defaultAccent = window.palette().color(QPalette::Highlight);
    accent->setCurrentIndex(accent->findData("cobalt"));
    QTRY_VERIFY(window.palette().color(QPalette::Highlight) != defaultAccent);
    accent->setCurrentIndex(accent->findData("custom"));
    auto *customAccent = window.findChild<QLineEdit *>("appearanceAccentColor");
    QTRY_VERIFY(customAccent && customAccent->isVisible());
    customAccent->setText("#1F6E99");
    QTest::keyClick(customAccent, Qt::Key_Return);
    QTRY_COMPARE(window.palette().color(QPalette::Highlight), QColor("#1F6E99"));
    background->setText("#102435");
    QTest::keyClick(background, Qt::Key_Return);
    QTRY_COMPARE(window.palette().color(QPalette::Window), QColor("#102435"));
    auto *foreground = window.findChild<QLineEdit *>("appearanceForeground");
    QVERIFY(foreground);
    foreground->setText("#DEF0F9");
    QTest::keyClick(foreground, Qt::Key_Return);
    QTRY_COMPARE(window.palette().color(QPalette::WindowText), QColor("#DEF0F9"));
    auto *backgroundPicker = window.findChild<QToolButton *>("appearanceBackgroundPicker");
    QVERIFY(backgroundPicker);
    QTest::mouseClick(backgroundPicker, Qt::LeftButton);
    auto *picker = backgroundPicker->findChild<QDialog *>("appearanceColorPicker");
    QTRY_VERIFY(picker && picker->isVisible());
    auto *hue = picker->findChild<QWidget *>("appearanceColorHue");
    auto *accept = picker->findChild<QPushButton *>("applyAppearanceColor");
    QVERIFY(hue && accept);
    interaction_cases::capture(*picker, "settings-background-color-picker");
    QTest::mouseClick(hue, Qt::LeftButton, Qt::NoModifier, QPoint(hue->width() * 3 / 4, hue->height() / 2));
    QTest::keyClick(hue, Qt::Key_Escape);
    QTRY_VERIFY(!backgroundPicker->findChild<QDialog *>("appearanceColorPicker"));
    QCOMPARE(window.palette().color(QPalette::Window), QColor("#102435"));
    QTest::mouseClick(backgroundPicker, Qt::LeftButton);
    picker = backgroundPicker->findChild<QDialog *>("appearanceColorPicker");
    QTRY_VERIFY(picker && picker->isVisible());
    hue = picker->findChild<QWidget *>("appearanceColorHue");
    accept = picker->findChild<QPushButton *>("applyAppearanceColor");
    QVERIFY(hue && accept);
    QTest::mouseClick(hue, Qt::LeftButton, Qt::NoModifier, QPoint(hue->width() * 3 / 4, hue->height() / 2));
    QTest::mouseClick(accept, Qt::LeftButton);
    QTRY_VERIFY(window.palette().color(QPalette::Window) != QColor("#102435"));
    background->clear();
    QTest::keyClick(background, Qt::Key_Return);
    QTRY_COMPARE(background->text(), window.palette().color(QPalette::Window).name().toUpper());
    reset->click();
    QTRY_COMPARE(window.palette().color(QPalette::Highlight), defaultAccent);
    QVERIFY(!motion->isChecked());
    QVERIFY(!reset->isVisible());

    const QSize originalSize = window.size();
    window.resize(window.minimumWidth(), 400);
    QTRY_VERIFY(system->mapTo(&window, QPoint{}).x() >= pages->mapTo(&window, QPoint{}).x());
    QTRY_VERIFY(dark->mapTo(&window, QPoint(dark->width(), 0)).x() <= window.width());
    window.resize(originalSize);

    auto *density = window.findChild<QSpinBox *>("densityChoice");
    auto *interfaceSize = window.findChild<QSpinBox *>("textScale");
    auto *conversationSize = window.findChild<QSpinBox *>("contentScaleChoice");
    auto *language = window.findChild<QComboBox *>("languageChoice");
    auto *font = window.findChild<QComboBox *>("uiFontChoice");
    QVERIFY(density && interfaceSize && conversationSize && language && font);
    QTest::keyClick(advanced, Qt::Key_Space);
    QVERIFY(density->isVisible());
    for (const auto *size : {density, interfaceSize, conversationSize, code})
        QCOMPARE(size->suffix(), QStringLiteral(" px"));
    interfaceSize->setValue(19);
    QTRY_COMPARE(navigation->font().pixelSize(), 19);
    QVERIFY(navigation->width() >= navigation->fontMetrics().horizontalAdvance(navigation->item(2)->text()) + 40);
    QVERIFY(background->width() >= background->fontMetrics().horizontalAdvance(background->text()) + 12);
    QCOMPARE(conversationSize->value(), 13);
    QCOMPARE(language->count(), 3);
    QCOMPARE(language->currentData().toString(), QStringLiteral("en"));
    QVERIFY(language->findData("vi") >= 0 && language->findData("zh_CN") >= 0);
    QVERIFY(font->count() >= 2);
    if (QGuiApplication::platformName() != "offscreen")
        QVERIFY(font->count() > 2);
    if (font->count() > 2) {
        const auto installedFamily = font->itemText(2);
        font->setCurrentIndex(2);
        QTRY_COMPARE(QApplication::font().family(), installedFamily);
        font->setCurrentIndex(0);
    }
    if (!qEnvironmentVariableIsEmpty("CGACP_UI_CAPTURE")) {
        auto *scroll = qobject_cast<QScrollArea *>(pages->currentWidget());
        QVERIFY(scroll);
        scroll->ensureWidgetVisible(language, 8, 24);
        interaction_cases::capture(window, "settings-appearance-pixels");
    }
    accent->showPopup();
    QTRY_VERIFY(accent->view()->isVisible());
    QCOMPARE(accent->view()->font().pixelSize(), 19);
    QTest::keyClick(accent->view(), Qt::Key_Escape);
    QTRY_VERIFY(!accent->view()->isVisible());
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
        QVERIFY(capability->text().contains("HANDOFF ONLY"));
    QTRY_VERIFY(!QApplication::activePopupWidget());
    QTRY_VERIFY(!QApplication::activeModalWidget());
    if (QGuiApplication::platformName() == "offscreen")
        window.activateWindow();
    QTRY_COMPARE(QApplication::activeWindow(), &window);
    QTest::keyClick(navigation, Qt::Key_Escape);
    QTRY_VERIFY(!navigation->isVisible());
}

template <class Request>
void verifyRuntimeSettingsData(MainWindow &window, const QString &bindingId, Request request)
{
    auto *accounting = window.findChild<QLabel *>("storageAccounting");
    auto *activeUsage = window.findChild<QLabel *>("storageActiveUsage");
    auto *archivedUsage = window.findChild<QLabel *>("storageArchivedUsage");
    QVERIFY(accounting && activeUsage && archivedUsage);
    QTRY_VERIFY(!activeUsage->text().isEmpty() && !archivedUsage->text().isEmpty());

    const QString title = QStringLiteral("Archived Settings fixture");
    const auto created = request("conversation.create", {{"title", title}, {"bindings", QJsonArray{bindingId}},
        {"profile", "handoff"}, {"key", "archived-settings-fixture"}});
    const auto conversation = created.value("conversation").isObject()
        ? created.value("conversation").toObject() : created;
    const auto id = conversation.value("id");
    QVERIFY(!id.toString().isEmpty());
    QVERIFY(!request("conversation.archive", {{"id", id}, {"revision", conversation.value("revision")}}).isEmpty());
    const auto duplicateReply = request("conversation.create", {{"title", title}, {"bindings", QJsonArray{bindingId}},
        {"profile", "handoff"}, {"key", "archived-settings-duplicate"}});
    const auto duplicate = duplicateReply.value("conversation").isObject()
        ? duplicateReply.value("conversation").toObject() : duplicateReply;
    const auto duplicateId = duplicate.value("id");
    QVERIFY(!duplicateId.toString().isEmpty() && duplicateId != id);
    QVERIFY(!request("conversation.archive", {{"id", duplicateId}, {"revision", duplicate.value("revision")}}).isEmpty());

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
    auto *chats = window.findChild<QWidget *>("archivedChats");
    auto *source = window.findChild<QComboBox *>("archivedSourceFilter");
    QVERIFY(search && chats && source);
    const auto card = [chats](const QJsonValue &conversationId) -> QWidget * {
        for (auto *row : chats->findChildren<QWidget *>("settingsCard", Qt::FindDirectChildrenOnly)) {
            if (row->property("conversationId").toString() == conversationId.toString())
                return row;
        }
        return nullptr;
    };
    search->setText(title);
    QTRY_VERIFY(card(id) && card(duplicateId));
    source->setCurrentIndex(source->findData("AUTOMATION"));
    QTRY_VERIFY(!card(id) && !card(duplicateId));
    source->setCurrentIndex(source->findData("DASHBOARD"));
    QTRY_VERIFY(card(id) && card(duplicateId));
    interaction_cases::capture(window, "archived-chats-real-runtime");
    auto *unarchive = card(id)->findChild<QPushButton *>("unarchiveChat");
    QVERIFY(unarchive && unarchive->isEnabled());
    unarchive->setFocus();
    QTest::keyClick(unarchive, Qt::Key_Space);
    QTRY_VERIFY(!card(id) && card(duplicateId));
    const auto active = request("conversation.list", {{"state", "ACTIVE"}, {"query", title}}).value("items").toArray();
    QCOMPARE(active.size(), 1);
    QCOMPARE(active[0].toObject().value("id"), id);
    QVERIFY(!request("conversation.archive", {{"id", id}, {"revision", active[0].toObject().value("revision")}}).isEmpty());
    auto *refresh = window.findChild<QPushButton *>("archivedRefresh");
    QVERIFY(refresh);
    refresh->click();
    QTRY_VERIFY(card(id) && card(duplicateId));
    auto *remove = card(id)->findChild<QToolButton *>("deleteArchivedChat");
    QVERIFY(remove && remove->isEnabled());
    remove->click();
    QTRY_VERIFY(window.findChild<QMessageBox *>("deleteDataConfirmation"));
    auto *confirmation = window.findChild<QMessageBox *>("deleteDataConfirmation");
    QVERIFY(confirmation->text().contains(title));
    confirmation->button(QMessageBox::Yes)->click();
    QTRY_VERIFY(!card(id) && card(duplicateId));
    const auto archived = request("conversation.list", {{"state", "ARCHIVED"}, {"query", title}}).value("items").toArray();
    QCOMPARE(archived.size(), 1);
    QCOMPARE(archived[0].toObject().value("id"), duplicateId);

    selectPage("storage");
    auto *refreshStorage = window.findChild<QPushButton *>("refreshChatStorage");
    auto *maintenance = window.findChild<QPushButton *>("storageMaintenanceToggle");
    auto *status = window.findChild<QLabel *>("storageOperationStatus");
    auto *backup = window.findChild<QPushButton *>("createStorageBackup");
    auto *verify = window.findChild<QPushButton *>("verifyStorageBackup");
    auto *backupSelector = window.findChild<QComboBox *>("storageBackups");
    QVERIFY(refreshStorage && maintenance && status && backup && verify && backupSelector);
    QTRY_VERIFY(refreshStorage->isEnabled());
    QVERIFY(accounting->text().contains("message text"));
    QVERIFY(!backup->isVisible());
    interaction_cases::capture(window, "storage-real-runtime");
    refreshStorage->click();
    QTRY_VERIFY(refreshStorage->isEnabled());
    maintenance->click();
    QVERIFY(backup->isVisible());
    backup->click();
    QTRY_VERIFY(backup->isEnabled() && status->text().contains("completed"));
    QVERIFY(!backupSelector->currentData().toString().isEmpty());
    verify->click();
    QTRY_VERIFY(verify->isEnabled() && status->text().contains("completed"));

    selectPage("configuration");
    auto *capability = window.findChild<QLabel *>("configurationCapability");
    QVERIFY(capability && capability->text().contains("HANDOFF ONLY"));
    interaction_cases::capture(window, "configuration-real-runtime");
}
