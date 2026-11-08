#pragma once

#include "main_window.h"

#include <QAction>
#include <QApplication>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMenu>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPointer>
#include <QPushButton>
#include <QScrollArea>
#include <QTableWidget>
#include <QTest>
#include <QTimeEdit>
#include <QTimer>
#include <QToolButton>

inline void verifyDashboardPages(MainWindow &window)
{
    auto *viewport = window.findChild<QScrollArea *>("automationViewport");
    QVERIFY(viewport);
    auto *active = window.findChild<QWidget *>("activeAutomationSection");
    QVERIFY(active);
    QTRY_COMPARE(active->findChildren<QLabel *>("automationCardTitle").size(), 2);
    const auto titles = active->findChildren<QLabel *>("automationCardTitle");
    const QPointer<QLabel> card(titles[0]);
    const QPointer<QLabel> second(titles[1]);
    const auto oldWidth = viewport->viewport()->width();
    const auto stacked = [&] {
        return card && second && card->parentWidget()->x() == second->parentWidget()->x()
            && card->parentWidget()->y() < second->parentWidget()->y();
    };
    if (oldWidth >= 710) {
        QTRY_VERIFY(card && second && card->parentWidget()->y() == second->parentWidget()->y()
                    && card->parentWidget()->x() < second->parentWidget()->x());
    } else {
        QTRY_VERIFY(stacked());
    }
    window.resize(800, 620);
    QTRY_VERIFY(viewport->viewport()->width() < oldWidth);
    QTRY_VERIFY(stacked());
    QVERIFY(card && card->isVisible());
    QCOMPARE(active->findChild<QLabel *>("automationCardTitle"), card.data());
    auto *refresh = window.findChild<QPushButton *>("refreshAutomations");
    QVERIFY(refresh);
    QTest::mouseClick(refresh, Qt::LeftButton);
    QTest::qWait(60);
    QVERIFY(card && card->isVisible());
    QCOMPARE(active->findChild<QLabel *>("automationCardTitle"), card.data());

    auto *history = window.findChild<QListWidget *>("conversationList");
    auto *composer = window.findChild<QPlainTextEdit *>("conversationComposer");
    auto *transcript = window.findChild<QPlainTextEdit *>("conversationTranscript");
    auto *settings = window.findChild<QToolButton *>("navSettings");
    auto *back = window.findChild<QPushButton *>("settingsBack");
    QVERIFY(history && composer && transcript && settings && back);
    QTRY_COMPARE(history->count(), 2);
    QVERIFY(history->isVisible());
    QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                      history->visualItemRect(history->item(0)).center());
    QTRY_VERIFY(composer->isVisible() && !composer->isReadOnly());
    QTRY_VERIFY(transcript->toPlainText().contains(QStringLiteral("Saved fixture message")));
    QVERIFY(composer->height() >= 4 * composer->fontMetrics().lineSpacing());
    QVERIFY(composer->height() <= 8 * composer->fontMetrics().lineSpacing());
    QTest::keyClicks(composer, "Draft kept across navigation");
    QTest::mouseClick(settings, Qt::LeftButton);
    QTRY_VERIFY(back->isVisible() && !composer->isVisible());
    QTest::mouseClick(back, Qt::LeftButton);
    QTRY_VERIFY(composer->isVisible());
    QCOMPARE(composer->toPlainText(), QStringLiteral("Draft kept across navigation"));

    QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                      history->visualItemRect(history->item(1)).center());
    QTRY_VERIFY(!composer->isReadOnly());
    QVERIFY(composer->toPlainText().isEmpty());
    QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                      history->visualItemRect(history->item(0)).center());
    QTRY_COMPARE(composer->toPlainText(), QStringLiteral("Draft kept across navigation"));

    auto *newChat = window.findChild<QPushButton *>("navNewChat");
    QVERIFY(newChat);
    QTest::mouseClick(newChat, Qt::LeftButton);
    QTRY_VERIFY(window.findChild<QDialog *>("newConversationDialog"));
    auto *dialog = window.findChild<QDialog *>("newConversationDialog");
    auto *title = dialog->findChild<QLineEdit *>("newConversationTitle");
    auto *manage = dialog->findChild<QPushButton *>("manageRepositoryPicker");
    auto *repositories = dialog->findChild<QListWidget *>("repositoryPickerList");
    auto *section = window.findChild<QListWidget *>("settingsNavigation");
    QVERIFY(dialog->isVisible() && title && manage && repositories && section);
    QTRY_VERIFY(repositories->count() > 0);
    title->selectAll();
    QTest::keyClicks(title, "A title to keep");
    QTest::mouseClick(repositories->viewport(), Qt::LeftButton, Qt::NoModifier,
                      repositories->visualItemRect(repositories->item(0)).center());
    QCOMPARE(repositories->selectedItems().size(), 1);
    QTest::mouseClick(manage, Qt::LeftButton);
    QTRY_VERIFY(back->isVisible() && !dialog->isVisible());
    QVERIFY(section->currentItem());
    QCOMPARE(section->currentItem()->data(Qt::UserRole).toString(), QStringLiteral("repositories"));
    QTest::mouseClick(back, Qt::LeftButton);
    QTRY_VERIFY(dialog->isVisible());
    QTRY_COMPARE(repositories->selectedItems().size(), 1);
    QCOMPARE(title->text(), QStringLiteral("A title to keep"));
    QTest::keyClick(dialog, Qt::Key_Escape);
    QTRY_VERIFY(!window.findChild<QDialog *>("newConversationDialog"));
    QCOMPARE(composer->toPlainText(), QStringLiteral("Draft kept across navigation"));

    auto *navAutomations = window.findChild<QPushButton *>("navAutomations");
    auto *addAutomation = window.findChild<QPushButton *>("addAutomation");
    QVERIFY(navAutomations && addAutomation);
    QTest::mouseClick(navAutomations, Qt::LeftButton);
    QTRY_VERIFY(addAutomation->isVisible());
    QTest::mouseClick(addAutomation, Qt::LeftButton);
    QTRY_VERIFY(window.findChild<QDialog *>("automationEditor"));
    dialog = window.findChild<QDialog *>("automationEditor");
    auto *name = dialog->findChild<QLineEdit *>("automationName");
    auto *prompt = dialog->findChild<QPlainTextEdit *>("automationPrompt");
    auto *automationManage = dialog->findChild<QPushButton *>("manageRepositoryPicker");
    auto *automationBindings = dialog->findChild<QListWidget *>("repositoryPickerList");
    QVERIFY(name && prompt && automationManage && automationBindings);
    QTest::keyClicks(name, "Local fixture definition");
    QTest::keyClicks(prompt, "Keep this prompt in the form");
    QTRY_VERIFY(automationBindings->count() > 0);
    QTest::mouseClick(automationBindings->viewport(), Qt::LeftButton, Qt::NoModifier,
                      automationBindings->visualItemRect(automationBindings->item(0)).center());
    QCOMPARE(automationBindings->selectedItems().size(), 1);
    QTest::mouseClick(automationManage, Qt::LeftButton);
    QTRY_VERIFY(back->isVisible() && !dialog->isVisible());
    QVERIFY(section->currentItem());
    QCOMPARE(section->currentItem()->data(Qt::UserRole).toString(), QStringLiteral("repositories"));
    QTest::mouseClick(back, Qt::LeftButton);
    QTRY_VERIFY(dialog->isVisible());
    QTRY_COMPARE(automationBindings->selectedItems().size(), 1);
    QCOMPARE(name->text(), QStringLiteral("Local fixture definition"));
    QCOMPARE(prompt->toPlainText(), QStringLiteral("Keep this prompt in the form"));
    QComboBox *trigger = nullptr;
    for (auto *combo : dialog->findChildren<QComboBox *>()) {
        if (combo->findData("DAILY") >= 0)
            trigger = combo;
    }
    QVERIFY(trigger);
    trigger->setCurrentIndex(trigger->findData("DAILY"));
    auto *time = dialog->findChild<QTimeEdit *>();
    QTRY_VERIFY(time && time->isVisible());
    // The offscreen platform has no window manager to reactivate the restored dialog.
    if (QApplication::platformName() == QStringLiteral("offscreen"))
        QApplication::setActiveWindow(dialog);
    time->setFocus();
    QTRY_VERIFY(time->hasFocus());
    time->setCurrentSection(QDateTimeEdit::MinuteSection);
    QCOMPARE(time->currentSection(), QDateTimeEdit::MinuteSection);
    const auto beforeStep = time->time();
    QTest::keyClick(time, Qt::Key_Up);
    QCOMPARE(time->time(), beforeStep.addSecs(60));
    if (const auto capture = qEnvironmentVariable("CGACP_UI_CAPTURE"); !capture.isEmpty())
        QVERIFY(dialog->grab().save(capture + "/automation-daily-controls.png"));
    QTest::keyClick(dialog, Qt::Key_Escape);
    QTRY_VERIFY(!window.findChild<QDialog *>("automationEditor"));
    QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                      history->visualItemRect(history->item(0)).center());
    QTRY_VERIFY(composer->isVisible());
}

template <class Request>
void verifyLegacyAutomation(MainWindow &window, const QString &bindingId, const QString &sourceName, Request request)
{
    const QJsonArray paths{sourceName};
    const auto original = request("automation.create", {{"key", "desktop-legacy-definition"},
        {"name", "Existing template fixture"}, {"binding_id", bindingId}, {"profile", "handoff"},
        {"prompt", "Review the selected source"}, {"template_id", "scoped-review"}, {"read_paths", paths},
        {"enabled", false}, {"trigger", QJsonObject{{"kind", "MANUAL"}}}});
    const auto id = original.value("id").toString();
    QVERIFY(!id.isEmpty());
    QCOMPARE(original.value("template_id").toString(), QStringLiteral("scoped-review"));
    QCOMPARE(original.value("read_paths").toArray(), paths);
    const auto persisted = request("automation.list", {}).value("items").toArray();
    QCOMPARE(persisted.size(), 1);
    QCOMPARE(persisted.first().toObject().value("id").toString(), id);

    auto *nav = window.findChild<QPushButton *>("navAutomations");
    auto *refresh = window.findChild<QPushButton *>("refreshAutomations");
    QVERIFY(nav && refresh);
    QTest::mouseClick(nav, Qt::LeftButton);
    QTRY_VERIFY(refresh->isVisible() && refresh->isEnabled());
    QTest::keyClick(refresh, Qt::Key_Space);
    const auto getTitle = [&] {
        for (auto *title : window.findChildren<QLabel *>("automationCardTitle"))
            if (title->text() == QStringLiteral("Existing template fixture"))
                return title;
        return static_cast<QLabel *>(nullptr);
    };
    QTRY_VERIFY(getTitle());
    const QPointer<QWidget> card(getTitle()->parentWidget());
    auto *run = card->findChild<QPushButton *>("automationRun");
    auto *edit = card->findChild<QAction *>("editAutomationDefinition");
    const QPointer<QAction> history(card->findChild<QAction *>("showAutomationHistory"));
    QVERIFY(run && edit && history);

    bool confirmed = false;
    QTimer driver;
    driver.setInterval(15);
    QObject::connect(&driver, &QTimer::timeout, &window, [&] {
        if (auto *confirmation = qobject_cast<QMessageBox *>(QApplication::activeModalWidget());
            confirmation && confirmation->windowTitle() == QStringLiteral("Run inactive automation")) {
            confirmed = true;
            confirmation->button(QMessageBox::Yes)->click();
        }
    });
    QTimer::singleShot(2000, &driver, [&] {
        if (!confirmed)
            if (auto *dialog = qobject_cast<QDialog *>(QApplication::activeModalWidget()))
                dialog->reject();
    });
    driver.start();
    QTest::keyClick(run, Qt::Key_Space);
    driver.stop();
    QVERIFY(confirmed);
    QTRY_VERIFY(run->isEnabled());
    const auto runs = request("automation.runs", {{"id", id}}).value("items").toArray();
    QCOMPARE(runs.size(), 1);
    const auto runId = runs.first().toObject().value("id").toString();
    QVERIFY(!runId.isEmpty());

    edit->trigger();
    QTRY_VERIFY(window.findChild<QDialog *>("automationEditor"));
    auto *dialog = window.findChild<QDialog *>("automationEditor");
    auto *name = dialog->findChild<QLineEdit *>("automationName");
    auto *buttons = dialog->findChild<QDialogButtonBox *>();
    QVERIFY(name && buttons);
    auto *save = buttons->button(QDialogButtonBox::Save);
    QVERIFY(save);
    QCOMPARE(name->text(), QStringLiteral("Existing template fixture"));
    name->setText(QStringLiteral("Renamed template fixture"));
    QTRY_VERIFY(save->isEnabled());
    QTest::mouseClick(save, Qt::LeftButton);
    QTRY_VERIFY(!window.findChild<QDialog *>("automationEditor"));
    QTRY_VERIFY(card && card->findChild<QLabel *>("automationCardTitle")->text() == QStringLiteral("Renamed template fixture"));
    const auto definitions = request("automation.list", {}).value("items").toArray();
    QCOMPARE(definitions.size(), 1);
    const auto updated = definitions.first().toObject();
    QCOMPARE(updated.value("id").toString(), id);
    QCOMPARE(updated.value("template_id"), original.value("template_id"));
    QCOMPARE(updated.value("template_revision"), original.value("template_revision"));
    QCOMPARE(updated.value("read_paths").toArray(), paths);
    QCOMPARE(updated.value("enabled").toBool(), false);

    QString retainedRunId;
    QTimer historyDriver;
    historyDriver.setInterval(15);
    QObject::connect(&historyDriver, &QTimer::timeout, &window, [&] {
        if (auto *historyDialog = qobject_cast<QDialog *>(QApplication::activeModalWidget()); historyDialog) {
            auto *table = historyDialog->findChild<QTableWidget *>();
            if (table && table->rowCount() == 1 && table->item(0, 0)) {
                retainedRunId = table->item(0, 0)->text();
                historyDialog->reject();
            }
        }
    });
    QTimer::singleShot(2000, &historyDriver, [&] {
        if (auto *historyDialog = qobject_cast<QDialog *>(QApplication::activeModalWidget()))
            historyDialog->reject();
    });
    historyDriver.start();
    QVERIFY(history);
    history->trigger();
    QTRY_COMPARE(retainedRunId, runId);

    auto *menuButton = card->findChild<QPushButton *>("automationActions");
    QVERIFY(menuButton && menuButton->menu());
    QAction *toggle = nullptr;
    for (auto *action : menuButton->menu()->actions()) {
        if (action->text() == "Enable")
            toggle = action;
    }
    QVERIFY(toggle && toggle->isEnabled());
    toggle->trigger();
    QTRY_COMPARE(toggle->text(), QString("Pause"));
    QVERIFY(request("automation.list", {}).value("items").toArray()[0].toObject().value("enabled").toBool());
    toggle->trigger();
    QTRY_COMPARE(toggle->text(), QString("Enable"));
    QVERIFY(!request("automation.list", {}).value("items").toArray()[0].toObject().value("enabled").toBool());
    const auto afterPause = request("automation.runs", {{"id", id}}).value("items").toArray();
    QCOMPARE(afterPause.size(), 1);
    QCOMPARE(afterPause.first().toObject().value("id").toString(), runId);
}
