#pragma once

#include <QApplication>
#include <QDebug>
#include <QElapsedTimer>
#include <QJsonArray>
#include <QJsonObject>
#include <QListWidget>
#include <QPaintEvent>
#include <QPlainTextEdit>
#include <QTest>
#include <QTextBlock>
#include <QTextCursor>
#include <QTimer>
#ifdef Q_OS_WIN
#include <qt_windows.h>
#include <psapi.h>
#include <tlhelp32.h>
#endif

#include <algorithm>
#include <cmath>
#include <utility>

namespace async_performance {

inline QString marker(const QString &id)
{
    return QStringLiteral("Transcript marker %1").arg(id);
}

inline QString multilineFixtureBody()
{
    return QStringLiteral("Repeated Unicode Ánh sáng 日本語.\n").repeated(420);
}

inline QJsonObject fixtureDetail(const QString &id, int lineBytes)
{
    const auto row = [](int ordinal, QString content) {
        return QJsonObject{{"id", QString::number(ordinal)}, {"ordinal", ordinal},
                           {"role", "USER"}, {"content", std::move(content)}};
    };
    const QString title = QStringLiteral("Performance fixture %1").arg(id);
    QJsonArray messages;
    if (lineBytes > 0) {
        messages.append(row(1, QString(lineBytes, u'w')));
        messages.append(row(2, multilineFixtureBody()));
    }
    messages.append(row(3, marker(id)));
    return {{"conversation", QJsonObject{{"id", id}, {"title", title}, {"state", "ACTIVE"}, {"revision", 1}}},
            {"messages", messages}};
}

// Observe completion of the actual viewport paint handler without forcing a repaint or grabbing the widget.
class PaintProbe {
public:
    PaintProbe(QPlainTextEdit *transcript, const QElapsedTimer &clock) : transcript_(transcript), clock_(clock) {}

    QWidget *viewport() const { return transcript_->viewport(); }

    void arm(const QString &expectedMarker)
    {
        expectedMarker_ = expectedMarker;
        start_ = clock_.nsecsElapsed();
        elapsed_ = -1;
    }

    bool matches(const QPaintEvent &event) const
    {
        if (elapsed_ >= 0 || expectedMarker_.isEmpty())
            return false;
        const auto block = transcript_->document()->lastBlock();
        if (block.length() != expectedMarker_.size() + 1 || block.text() != expectedMarker_)
            return false;
        return event.region().intersects(transcript_->cursorRect(QTextCursor(block)));
    }

    void completed() { elapsed_ = clock_.nsecsElapsed() - start_; }
    double ms() const { return elapsed_ / 1.0e6; }

private:
    QPlainTextEdit *transcript_;
    const QElapsedTimer &clock_;
    QString expectedMarker_;
    qint64 start_ = 0;
    qint64 elapsed_ = -1;
};

class Application final : public QApplication {
public:
    using QApplication::QApplication;

    bool notify(QObject *receiver, QEvent *event) override
    {
        const bool measure = event->type() == QEvent::Paint && probe && receiver == probe->viewport()
            && probe->matches(*static_cast<QPaintEvent *>(event));
        const bool accepted = QApplication::notify(receiver, event);
        if (measure)
            probe->completed();
        return accepted;
    }

    PaintProbe *probe = nullptr;
};

struct ScopedProbe {
    explicit ScopedProbe(PaintProbe &probe) { static_cast<Application *>(qApp)->probe = &probe; }
    ~ScopedProbe() { static_cast<Application *>(qApp)->probe = nullptr; }
};

inline void summarize(const char *series, QList<double> values)
{
    if (values.isEmpty()) {
        qInfo() << "async_perf" << series << "n 0";
        return;
    }
    std::sort(values.begin(), values.end());
    const auto quantile = [&values](double fraction) {
        return values[qsizetype(std::ceil(values.size() * fraction)) - 1];
    };
    qInfo() << "async_perf" << series << "n" << values.size() << "ms p50 p95 p99 max"
            << quantile(0.50) << quantile(0.95) << quantile(0.99) << values.last();
}

inline void clickRow(QListWidget *list, int index)
{
    QTest::mouseClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                      list->visualItemRect(list->item(index)).center());
}

inline int detailCount(const QJsonArray &methods)
{
    return int(std::count(methods.begin(), methods.end(), QJsonValue("conversation.get")));
}

inline void processSnapshot(const char *phase, qint64 runtimePid)
{
#ifdef Q_OS_WIN
    const auto capture = [phase](const char *role, DWORD pid) {
        const HANDLE process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);
        if (!process) {
            qInfo() << "async_perf process" << phase << role << "unavailable";
            return;
        }
        PROCESS_MEMORY_COUNTERS_EX memory{};
        const bool haveMemory = K32GetProcessMemoryInfo(process, reinterpret_cast<PROCESS_MEMORY_COUNTERS *>(&memory),
                                                       sizeof memory);
        DWORD handles = 0;
        GetProcessHandleCount(process, &handles);
        FILETIME created{}, exited{}, kernel{}, user{};
        GetProcessTimes(process, &created, &exited, &kernel, &user);
        const auto millis = [](const FILETIME &time) {
            return ((quint64(time.dwHighDateTime) << 32) | time.dwLowDateTime) / 10000.0;
        };
        CloseHandle(process);

        int threads = -1;
        int children = -1;
        const HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if (snapshot != INVALID_HANDLE_VALUE) {
            PROCESSENTRY32W entry{};
            entry.dwSize = sizeof entry;
            children = 0;
            for (BOOL more = Process32FirstW(snapshot, &entry); more; more = Process32NextW(snapshot, &entry)) {
                if (entry.th32ProcessID == pid)
                    threads = int(entry.cntThreads);
                if (entry.th32ParentProcessID == pid)
                    ++children;
            }
            CloseHandle(snapshot);
        }
        qInfo() << "async_perf process" << phase << role << "private_bytes working_set cpu_ms handles threads children"
                << (haveMemory ? qint64(memory.PrivateUsage) : -1) << (haveMemory ? qint64(memory.WorkingSetSize) : -1)
                << (millis(kernel) + millis(user)) << handles << threads << children;
    };
    capture("gui", GetCurrentProcessId());
    if (runtimePid > 0)
        capture("fixture", DWORD(runtimePid));
#else
    Q_UNUSED(phase);
    Q_UNUSED(runtimePid);
#endif
}

} // namespace async_performance
