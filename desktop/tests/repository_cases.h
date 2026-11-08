#pragma once

#include "main_window.h"

#include <QAction>
#include <QApplication>
#include <QCheckBox>
#include <QClipboard>
#include <QDesktopServices>
#include <QDir>
#include <QFileInfo>
#include <QHeaderView>
#include <QInputDialog>
#include <QLabel>
#include <QLineEdit>
#include <QMessageBox>
#include <QMouseEvent>
#include <QPushButton>
#include <QSignalSpy>
#include <QTableView>
#include <QTest>
#include <QTimer>
#include <QUrl>

namespace repository_cases {
class FolderProbe final : public QObject {
    Q_OBJECT
public:
    FolderProbe() { QDesktopServices::setUrlHandler("file", this, "record"); }
    ~FolderProbe() override { QDesktopServices::unsetUrlHandler("file"); }

    QUrl opened;

public slots:
    void record(const QUrl &url) { opened = url; }
};

inline QJsonObject selectedBinding(const QTableView *table)
{
    const auto rows = table->selectionModel()->selectedRows();
    return rows.isEmpty() ? QJsonObject() : rows.first().data(Qt::UserRole + 1).toJsonObject();
}

inline void exerciseTable(MainWindow &window, const QJsonObject &binding)
{
    auto *table = window.findChild<QTableView *>("repositoriesTable");
    auto *search = window.findChild<QLineEdit *>("repoSearch");
    auto *copy = window.findChild<QAction *>("repoCopyPath");
    auto *open = window.findChild<QAction *>("repoOpenFolder");
    auto *health = window.findChild<QAction *>("repoCheckHealth");
    auto *refresh = window.findChild<QPushButton *>("repoRefresh");
    auto *summary = window.findChild<QLabel *>("repoSummary");
    auto *details = window.findChild<QLabel *>("repoDetails");
    auto *rename = window.findChild<QAction *>("repoRename");
    QVERIFY(table && search && copy && open && health && refresh && summary && details && rename);
    QVERIFY(table->isVisible());
    QTRY_COMPARE(table->model()->rowCount(), 1);
    const auto id = binding.value("binding_id").toString();
    const auto root = binding.value("root").toString();
    QVERIFY(!id.isEmpty() && !root.isEmpty());

    auto *header = table->horizontalHeader();
    for (int i = 0; i < header->count(); ++i)
        QCOMPARE(header->sectionResizeMode(i), QHeaderView::Interactive);
    const auto initialWidth = header->sectionSize(1);
    const QPoint handle(header->sectionViewportPosition(1) + initialWidth - 1, header->height() / 2);
    QVERIFY(header->viewport()->rect().contains(handle));
    const auto target = handle + QPoint(42, 0);
    QTest::mousePress(header->viewport(), Qt::LeftButton, Qt::NoModifier, handle);
    QMouseEvent drag(QEvent::MouseMove, target, header->viewport()->mapToGlobal(target),
                     Qt::NoButton, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(header->viewport(), &drag);
    QTest::mouseRelease(header->viewport(), Qt::LeftButton, Qt::NoModifier, target);
    const auto resizedWidth = header->sectionSize(1);
    QVERIFY(resizedWidth > initialWidth);

    table->selectRow(0);
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);
    const auto path = table->model()->index(0, 1);
    QVERIFY(path.data(Qt::ToolTipRole).toString().contains(root.toHtmlEscaped()));
#ifdef Q_OS_WIN
    if (root.startsWith(QStringLiteral(R"(\\?\)")))
        QCOMPARE(path.data().toString(), root.mid(4));
#endif
    QVERIFY(copy->isEnabled());
    copy->trigger();
    if (qEnvironmentVariableIntValue("CGACP_SKIP_NATIVE_CLIPBOARD") == 1)
        qInfo("Native clipboard comparison unavailable in this sandbox. Offscreen run verifies the full path.");
    else
        QCOMPARE(QApplication::clipboard()->text(), root);
    {
        FolderProbe probe;
        QVERIFY(open->isEnabled());
        open->trigger();
        QTRY_VERIFY(probe.opened.isLocalFile());
        QCOMPARE(QDir::cleanPath(probe.opened.toLocalFile()), QDir::cleanPath(path.data().toString()));
    }

    QTest::mouseClick(refresh, Qt::LeftButton);
    QTRY_VERIFY(!summary->text().contains(QStringLiteral("Loading")));
    QCOMPARE(header->sectionSize(1), resizedWidth);
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);

    QSignalSpy fit(header, &QHeaderView::sectionHandleDoubleClicked);
    const QPoint widerHandle(header->sectionViewportPosition(1) + resizedWidth - 1, header->height() / 2);
    QTest::mouseDClick(header->viewport(), Qt::LeftButton, Qt::NoModifier, widerHandle);
    QTRY_COMPARE(fit.size(), 1);
    QVERIFY(header->sectionSize(1) > 0);

    const QPoint nameColumn(header->sectionViewportPosition(0) + header->sectionSize(0) / 2,
                            header->height() / 2);
    QTest::mouseClick(header->viewport(), Qt::LeftButton, Qt::NoModifier, nameColumn);
    QCOMPARE(header->sortIndicatorSection(), 0);
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);
    search->setFocus();
    QTest::keyClicks(search, "no-matching-binding");
    QTRY_COMPARE(table->model()->rowCount(), 0);
    QVERIFY(!rename->isEnabled());
    QTest::keyClick(search, Qt::Key_A, Qt::ControlModifier);
    QTest::keyClick(search, Qt::Key_Backspace);
    QTRY_COMPARE(table->model()->rowCount(), 1);
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);

    QVERIFY(health->isEnabled());
    health->trigger();
    QTRY_VERIFY(details->text().contains(QStringLiteral("Health checked")));
    QCOMPARE(table->model()->index(0, 3).data().toString(), QStringLiteral("Verified"));
    QTRY_VERIFY(rename->isEnabled());

    const auto renamed = QStringLiteral("Binding kiểm thử đã đổi tên");
    QTimer::singleShot(0, &window, [renamed] {
        if (auto *dialog = qobject_cast<QInputDialog *>(QApplication::activeModalWidget())) {
            dialog->setTextValue(renamed);
            dialog->accept();
        }
    });
    rename->trigger();
    QTRY_COMPARE(table->model()->index(0, 0).data().toString(), renamed);
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);
    QCOMPARE(selectedBinding(table).value("repo_id"), binding.value("repo_id"));
}

inline void exerciseArchive(MainWindow &window, const QString &id, const QString &source)
{
    auto *table = window.findChild<QTableView *>("repositoriesTable");
    auto *archive = window.findChild<QAction *>("repoArchive");
    auto *archived = window.findChild<QCheckBox *>("repoShowArchived");
    QVERIFY(table && archive && archived);
    QTRY_COMPARE(table->model()->rowCount(), 1);
    table->selectRow(0);
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);
    QVERIFY(archive->isEnabled());
    QTimer::singleShot(0, &window, [] {
        if (auto *dialog = qobject_cast<QMessageBox *>(QApplication::activeModalWidget()))
            dialog->button(QMessageBox::Yes)->click();
    });
    archive->trigger();
    QTRY_COMPARE(table->model()->rowCount(), 0);
    QVERIFY(QFileInfo::exists(source));
    archived->setFocus();
    QTest::keyClick(archived, Qt::Key_Space);
    QTRY_COMPARE(table->model()->rowCount(), 1);
    QCOMPARE(table->model()->index(0, 3).data().toString(), QStringLiteral("Archived"));
    QCOMPARE(selectedBinding(table).value("binding_id").toString(), id);
    QVERIFY(!archive->isEnabled());
}
}
