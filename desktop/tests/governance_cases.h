#pragma once

#include "main_window.h"
#include "live_text.h"

#include <QDialog>
#include <QElapsedTimer>
#include <QLineEdit>
#include <QLabel>
#include <QListWidget>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QTableView>
#include <QTemporaryDir>
#include <QTabWidget>
#include <QScrollArea>
#include <QToolButton>
#include <QTimer>
#include <QtTest>
#include <algorithm>

namespace governance_cases {
inline void authorSkill(const std::function<void(MainWindow &, const QString &)> &capture,
                        const std::function<void(MainWindow &)> &openConfiguration)
{
    const auto runtime = qEnvironmentVariable("CIRNETRULES_TEST_RUNTIME");
    if (runtime.isEmpty())
        QSKIP("Set CIRNETRULES_TEST_RUNTIME to exercise native authoring against Rust.");
    QTemporaryDir fixture;
    QVERIFY(fixture.isValid());
    const QStringList args{"--stdio", "--data-dir", fixture.filePath("data")};
    const auto edit = [](MainWindow &window, const char *name) {
        return window.findChild<QPlainTextEdit *>(QString::fromLatin1(name));
    };
    const auto button = [](MainWindow &window, const char *name) {
        return window.findChild<QPushButton *>(QString::fromLatin1(name));
    };
    const auto settleAndOpen = [&](MainWindow &window) {
        QTRY_VERIFY(window.findChild<QToolButton *>("navSettings")->isEnabled());
        openConfiguration(window);
        QTRY_VERIFY(button(window, "governanceNew")->isEnabled());
    };
    {
        RuntimeClient client(runtime, args);
        MainWindow window(client);
        window.resize(1240, 740);
        window.show();
        client.start();
        settleAndOpen(window);
        auto *create = button(window, "governanceNew");
        QTimer::singleShot(0, &window, [] {
            auto *dialog = qobject_cast<QDialog *>(QApplication::activeModalWidget());
            QVERIFY(dialog);
            dialog->findChild<QLineEdit *>("governanceNewTitle")->setText("Review fixture patch");
            dialog->findChild<QLineEdit *>("governanceNewSlug")->setText("review-fixture-patch");
            dialog->accept();
        });
        QTest::mouseClick(create, Qt::LeftButton);
        auto *english = edit(window, "governanceEnglish");
        auto *vietnamese = edit(window, "governanceVietnamese");
        auto *save = button(window, "governanceSave");
        QTRY_VERIFY(english->isEnabled() && !english->isReadOnly());
        english->setPlainText("---\nname: review-fixture-patch\ndescription: Review fixture changes\n---\n# Review fixture\nCheck the scoped diff.");
        vietnamese->setPlainText(QString::fromUtf8("# Rà soát fixture\nKiểm tra diff đã chọn."));
        auto *filesTab = window.findChild<QTabWidget *>("governanceEditorTabs");
        QVERIFY(filesTab);
        filesTab->setCurrentIndex(2);
        QTimer::singleShot(0, &window, [] {
            auto *dialog = qobject_cast<QDialog *>(QApplication::activeModalWidget());
            QVERIFY(dialog);
            dialog->findChild<QLineEdit *>("governanceFilePath")->setText("references/review.md");
            dialog->findChild<QPlainTextEdit *>("governanceFileContent")->setPlainText("# Review notes");
            dialog->accept();
        });
        QTest::mouseClick(button(window, "governanceAddFile"), Qt::LeftButton);
        QTRY_VERIFY(save->isEnabled());
        QTest::mouseClick(save, Qt::LeftButton);
        auto *validate = button(window, "governanceValidate");
        QTRY_VERIFY(validate->isEnabled());
        QTest::mouseClick(validate, Qt::LeftButton);
        auto *review = button(window, "governanceReview");
        QTRY_VERIFY(review->isEnabled());
        QTest::mouseClick(review, Qt::LeftButton);
        auto *apply = button(window, "governanceApply");
        QTRY_VERIFY(apply->isEnabled());
        capture(window, "governance-reviewed-diff");
        const auto approveNextModal = [&window] {
            QTimer::singleShot(0, &window, [] {
                auto *dialog = qobject_cast<QMessageBox *>(QApplication::activeModalWidget());
                QVERIFY(dialog);
                dialog->button(QMessageBox::Yes)->click();
            });
        };
        approveNextModal();
        QTest::mouseClick(apply, Qt::LeftButton);
        QTRY_VERIFY(button(window, "governanceHistory")->isEnabled());
        QTRY_VERIFY(window.findChild<QLabel *>("governanceStatus")->text().contains("published"));
        filesTab->setCurrentIndex(0);
        capture(window, "governance-published-skill");
        english->appendPlainText("Second published source");
        vietnamese->appendPlainText(QString::fromUtf8("Bản công bố thứ hai"));
        QTest::mouseClick(save, Qt::LeftButton);
        QTRY_VERIFY(review->isEnabled());
        QTest::mouseClick(review, Qt::LeftButton);
        QTRY_VERIFY(apply->isEnabled());
        approveNextModal();
        QTest::mouseClick(apply, Qt::LeftButton);
        auto *history = button(window, "governanceHistory");
        QTRY_VERIFY(history->isEnabled());
        QTimer::singleShot(0, &window, [&] {
            auto *dialog = qobject_cast<QDialog *>(QApplication::activeModalWidget());
            QVERIFY(dialog);
            auto *rows = dialog->findChild<QListWidget *>("governanceHistoryItems");
            QVERIFY(rows);
            QTRY_COMPARE(rows->count(), 2);
            rows->setCurrentRow(1);
            approveNextModal();
            QTest::mouseClick(dialog->findChild<QPushButton *>("governanceRollback"), Qt::LeftButton);
        });
        QTest::mouseClick(history, Qt::LeftButton);
        QTRY_VERIFY(!english->toPlainText().contains("Second published source"));
        QVERIFY(english->toPlainText().contains("Check the scoped diff."));
        client.stop();
        QTRY_VERIFY(!client.isRunning());
    }
    {
        RuntimeClient reopened(runtime, args);
        MainWindow window(reopened);
        window.show();
        reopened.start();
        settleAndOpen(window);
        auto *table = window.findChild<QTableView *>("governanceCatalog");
        QTRY_COMPARE(table->model()->rowCount(), 1);
        const auto first = table->model()->index(0, 0);
        QTest::mouseClick(table->viewport(), Qt::LeftButton, {}, table->visualRect(first).center());
        auto *english = edit(window, "governanceEnglish");
        QTRY_VERIFY(english->toPlainText().contains("Check the scoped diff."));
        QVERIFY(edit(window, "governanceVietnamese")->toPlainText().contains(QString::fromUtf8("Rà soát")));
        auto *files = window.findChild<QListWidget *>("governanceFiles");
        QCOMPARE(files->count(), 1);
        QCOMPARE(files->item(0)->text(), QStringLiteral("references/review.md"));
        english->appendPlainText("Unsaved source change");
        QTRY_VERIFY(window.findChild<QLabel *>("governanceStatus")->text().contains("Draft saved"));

        auto *navigation = window.findChild<QListWidget *>("settingsNavigation");
        const int governanceRow = navigation->currentRow();
        auto *configuration = window.findChild<QWidget *>("configurationPage");
        const auto count = configuration->findChildren<QObject *>().size();
        QList<qint64> transitions;
        QElapsedTimer timer;
        for (int cycle = 0; cycle < 100; ++cycle) {
            timer.start();
            navigation->setCurrentRow(0);
            navigation->setCurrentRow(governanceRow);
            QCoreApplication::processEvents();
            transitions.append(timer.nsecsElapsed() / 1000);
        }
        std::sort(transitions.begin(), transitions.end());
        QCOMPARE(configuration->findChildren<QObject *>().size(), count);
        qInfo() << "Governance view transition pair, 100 cycles, p50/p95/max us"
                << transitions[49] << transitions[94] << transitions.last()
                << "owned QObjects" << count;
        if (live::language().set("vi")) {
            QTRY_COMPARE(button(window, "governanceNew")->text(), QString::fromUtf8("Tạo tài nguyên"));
            QTest::qWait(120);
            capture(window, "governance-vietnamese");
            live::language().set("en");
        }
        window.resize(760, 520);
        QTRY_VERIFY(window.findChild<QPushButton *>("governanceBack")->isVisible());
        auto *inspector = window.findChild<QScrollArea *>("governanceInspector");
        QTRY_VERIFY(inspector->geometry().left() < 20 && !table->isVisible());
        QTest::qWait(120);
        capture(window, "governance-narrow-editor");
        QTest::mouseClick(button(window, "governanceBack"), Qt::LeftButton);
        QTRY_VERIFY(table->isVisible() && !inspector->isVisible());
        QTest::qWait(120);
        capture(window, "governance-narrow-catalog");
        QTest::mouseClick(table->viewport(), Qt::LeftButton, {}, table->visualRect(first).center());
        QTRY_VERIFY(button(window, "governanceBack")->isVisible());

        reopened.stop();
        QTRY_VERIFY(!reopened.isRunning());
    }
}
}
