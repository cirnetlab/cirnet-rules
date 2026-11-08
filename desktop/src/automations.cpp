#include "main_window.h"

#include <QCheckBox>
#include <QComboBox>
#include <QDateTime>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QLineEdit>
#include <QLocale>
#include <QMenu>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPointer>
#include <QPushButton>
#include <QScrollArea>
#include <QTableWidget>
#include <QTimeEdit>
#include <QTimeZone>
#include <QTimer>
#include <QUuid>
#include <QVBoxLayout>
#include <memory>

namespace {
QLabel *textLabel(const QString &text, const char *name)
{
    auto *label = new QLabel(text);
    label->setObjectName(QString::fromLatin1(name));
    label->setTextFormat(Qt::PlainText);
    label->setWordWrap(true);
    return label;
}

QString idempotencyKey()
{
    return QUuid::createUuid().toString(QUuid::WithoutBraces);
}

QString when(const QJsonValue &value)
{
    const auto seconds = value.toInteger();
    return seconds <= 0 ? QString() : QLocale().toString(
        QDateTime::fromSecsSinceEpoch(seconds, QTimeZone::systemTimeZone()), QLocale::ShortFormat);
}

void clearCards(QVBoxLayout *layout)
{
    while (auto *item = layout->takeAt(0)) {
        delete item->widget();
        delete item;
    }
}
}

QWidget *MainWindow::makeAutomations()
{
    auto *page = new QWidget;
    page->setObjectName("automationsPage");
    auto *outer = new QHBoxLayout(page);
    outer->setContentsMargins(36, 25, 36, 12);
    auto *panel = new QWidget;
    panel->setMaximumWidth(1060);
    auto *layout = new QVBoxLayout(panel);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(12);
    outer->addStretch();
    outer->addWidget(panel, 100);
    outer->addStretch();
    panel->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);

    auto *top = new QHBoxLayout;
    top->addWidget(textLabel(tr("Automations"), "heading"), 1);
    auto *add = new QPushButton(tr("+  Add Automation"));
    add->setObjectName("addAutomation");
    add->setProperty("primary", true);
    add->setAccessibleName(tr("Add Automation"));
    top->addWidget(add);
    layout->addLayout(top);
    layout->addWidget(textLabel(tr("Manage local schedules, inspect run history and configure reusable workflows."), "lead"));

    auto *filter = new QHBoxLayout;
    automationSearch_ = new QLineEdit;
    automationSearch_->setPlaceholderText(tr("Search automations…"));
    automationSearch_->setAccessibleName(tr("Search automations"));
    automationFilter_ = new QComboBox;
    automationFilter_->setAccessibleName(tr("Filter automation status"));
    automationFilter_->addItem(tr("All statuses"), QString());
    automationFilter_->addItem(tr("Active"), QStringLiteral("active"));
    automationFilter_->addItem(tr("Inactive"), QStringLiteral("inactive"));
    automationRepoFilter_ = new QComboBox;
    automationRepoFilter_->setAccessibleName(tr("Filter automation repository"));
    automationRepoFilter_->addItem(tr("All repositories"), QString());
    automationRepoFilter_->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
    automationRepoFilter_->setMinimumContentsLength(13);
    auto *refresh = new QPushButton(tr("Refresh"));
    filter->addWidget(automationSearch_, 1);
    filter->addWidget(automationFilter_);
    filter->addWidget(automationRepoFilter_);
    filter->addWidget(refresh);
    layout->addLayout(filter);

    auto *scroll = new QScrollArea;
    scroll->setWidgetResizable(true);
    scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    auto *contents = new QWidget;
    automationCards_ = new QVBoxLayout(contents);
    automationCards_->setContentsMargins(0, 4, 6, 0);
    automationCards_->setSpacing(12);
    scroll->setWidget(contents);
    layout->addWidget(scroll, 1);
    automationMore_ = new QPushButton(tr("Load more"));
    automationMore_->hide();
    layout->addWidget(automationMore_, 0, Qt::AlignRight);

    auto *searchDelay = new QTimer(page);
    searchDelay->setSingleShot(true);
    searchDelay->setInterval(250);
    connect(automationSearch_, &QLineEdit::textChanged, searchDelay, qOverload<>(&QTimer::start));
    connect(searchDelay, &QTimer::timeout, this, [this] { refreshAutomations(); });
    connect(automationFilter_, &QComboBox::currentIndexChanged, this, &MainWindow::renderAutomationCards);
    connect(automationRepoFilter_, &QComboBox::currentIndexChanged, this, &MainWindow::renderAutomationCards);
    connect(automationMore_, &QPushButton::clicked, this, [this] { refreshAutomations(true); });
    connect(refresh, &QPushButton::clicked, this, [this] {
        refreshAutomations();
        refreshTemplates();
    });
    connect(add, &QPushButton::clicked, this, [this] { editAutomation(); });
    renderAutomationCards();
    return page;
}

void MainWindow::refreshAutomations(bool append)
{
    if (!online_)
        return;
    QJsonObject params{{"limit", 20}, {"query", automationSearch_->text().trimmed()}};
    if (append && !automationCursor_.isNull() && !automationCursor_.isUndefined())
        params.insert("before", automationCursor_);
    const auto request = ++automationRequest_;
    automationMore_->setEnabled(false);
    call("automation.list", params, [this, request, append](const QJsonObject &result, const QString &error) {
        if (request != automationRequest_)
            return;
        automationMore_->setEnabled(true);
        if (!error.isEmpty()) {
            showStatus(error, true);
            return;
        }
        if (!append)
            automations_ = {};
        for (const auto &row : result.value("items").toArray())
            automations_.append(row);
        automationCursor_ = result.value("next_before");
        automationMore_->setVisible(!automationCursor_.isNull() && !automationCursor_.isUndefined());
        renderAutomationCards();
    });
}

void MainWindow::refreshTemplates()
{
    if (!online_)
        return;
    call("automation.templates", {}, [this](const QJsonObject &result, const QString &error) {
        if (!error.isEmpty()) {
            showStatus(error, true);
            return;
        }
        templates_ = result.value("items").toArray();
        renderAutomationCards();
    });
}

void MainWindow::renderAutomationCards()
{
    clearCards(automationCards_);
    cardColumns_ = width() >= 1120 ? 2 : 1;
    const auto bindingFilter = automationRepoFilter_->currentData().toString();
    const auto state = automationFilter_->currentData().toString();

    auto section = [this, &bindingFilter](bool enabled) {
        auto *wrapper = new QWidget;
        auto *stack = new QVBoxLayout(wrapper);
        stack->setContentsMargins(0, 0, 0, 0);
        auto *title = textLabel(QString(), "sectionTitle");
        stack->addWidget(title);
        auto *grid = new QGridLayout;
        grid->setContentsMargins(0, 0, 0, 0);
        grid->setSpacing(9);
        for (int column = 0; column < cardColumns_; ++column)
            grid->setColumnStretch(column, 1);
        int count = 0;
        for (const auto &value : automations_) {
            const auto item = value.toObject();
            const auto binding = item.value("binding_id").toString();
            if (item.value("enabled").toBool() != enabled || (!bindingFilter.isEmpty() && binding != bindingFilter))
                continue;
            auto *card = new QFrame;
            card->setObjectName("card");
            card->setMinimumHeight(124);
            auto *cardLayout = new QVBoxLayout(card);
            cardLayout->setContentsMargins(13, 11, 13, 11);
            cardLayout->setSpacing(6);
            auto *head = new QHBoxLayout;
            const auto name = item.value("name").toString();
            auto *nameLabel = textLabel(name.left(100), "automationCardTitle");
            nameLabel->setToolTip(name.toHtmlEscaped());
            head->addWidget(nameLabel, 1);
            auto *run = new QPushButton(tr("Run now"));
            run->setAccessibleName(tr("Run %1 now").arg(name));
            auto *more = new QPushButton(QStringLiteral("..."));
            more->setAccessibleName(tr("Actions for %1").arg(name));
            head->addWidget(run);
            head->addWidget(more);
            cardLayout->addLayout(head);
            auto *body = textLabel(item.value("prompt").toString().simplified().left(180), "cardBody");
            body->setMaximumHeight(42);
            body->setToolTip(item.value("prompt").toString().left(500).toHtmlEscaped());
            cardLayout->addWidget(body);
            const auto trigger = item.value("trigger").toObject();
            QString time = tr("Manual");
            if (trigger.value("kind").toString() == "DAILY") {
                const QTime slot(trigger.value("hour").toInt(), trigger.value("minute").toInt());
                time = tr("Daily %1 · %2").arg(slot.toString("HH:mm"), trigger.value("timezone").toString());
            }
            auto *tags = new QHBoxLayout;
            auto *badge = textLabel(time, enabled ? "chip" : "mutedChip");
            tags->addWidget(badge);
            const auto next = when(item.value("next_at"));
            if (!next.isEmpty())
                tags->addWidget(textLabel(tr("Next %1").arg(next), "meta"));
            tags->addStretch();
            cardLayout->addLayout(tags);
            const auto lastRun = item.value("last_run").toObject();
            if (!lastRun.isEmpty())
                cardLayout->addWidget(textLabel(tr("Last run: %1").arg(lastRun.value("status").toString()), "meta"));
            connect(run, &QPushButton::clicked, this, [this, item, run] {
                run->setEnabled(false);
                runAutomation(item);
            });
            connect(more, &QPushButton::clicked, this, [this, item, enabled, more] {
                QMenu menu(this);
                auto *edit = menu.addAction(tr("Edit definition"));
                auto *toggle = menu.addAction(enabled ? tr("Pause") : tr("Enable"));
                auto *history = menu.addAction(tr("Run history"));
                const auto selected = menu.exec(more->mapToGlobal(QPoint(0, more->height())));
                if (selected == edit)
                    editAutomation(item);
                else if (selected == toggle)
                    toggleAutomation(item);
                else if (selected == history)
                    automationHistory(item);
            });
            grid->addWidget(card, count / cardColumns_, count % cardColumns_);
            ++count;
        }
        title->setText((enabled ? tr("Active · %1") : tr("Inactive · %1")).arg(count));
        if (count == 0)
            stack->addWidget(textLabel(enabled ? tr("No active automations in this view.")
                                               : tr("No inactive automations in this view."), "meta"));
        else
            stack->addLayout(grid);
        automationCards_->addWidget(wrapper);
    };
    if (state != "inactive")
        section(true);
    if (state != "active")
        section(false);

    auto *gallery = new QWidget;
    auto *galleryLayout = new QVBoxLayout(gallery);
    galleryLayout->setContentsMargins(0, 2, 0, 0);
    galleryLayout->addWidget(textLabel(tr("Start from a workflow"), "sectionTitle"));
    galleryLayout->addWidget(textLabel(tr("Templates open a draft. Review their verification status before enabling a schedule."), "lead"));
    auto *grid = new QGridLayout;
    grid->setSpacing(9);
    for (int column = 0; column < cardColumns_; ++column)
        grid->setColumnStretch(column, 1);
    int count = 0;
    for (const auto &value : templates_) {
        const auto item = value.toObject();
        auto *card = new QFrame;
        card->setObjectName("card");
        auto *cardLayout = new QVBoxLayout(card);
        cardLayout->setContentsMargins(13, 10, 13, 10);
        auto *header = new QHBoxLayout;
        const auto name = item.value("title").toString();
        header->addWidget(textLabel(name, "automationTemplateTitle"), 1);
        auto *use = new QPushButton(QStringLiteral("+"));
        use->setAccessibleName(tr("Use template %1").arg(name));
        header->addWidget(use);
        cardLayout->addLayout(header);
        cardLayout->addWidget(textLabel(item.value("description").toString(), "cardBody"));
        cardLayout->addWidget(textLabel(tr("%1 · revision %2")
            .arg(item.value("status").toString(), QString::number(item.value("revision").toInt())), "meta"));
        connect(use, &QPushButton::clicked, this, [this, item] { editAutomation({}, item); });
        grid->addWidget(card, count / cardColumns_, count % cardColumns_);
        ++count;
    }
    if (count)
        galleryLayout->addLayout(grid);
    else
        galleryLayout->addWidget(textLabel(tr("Workflow templates appear when the runtime is connected."), "meta"));
    automationCards_->addWidget(gallery);
    automationCards_->addStretch();
}

void MainWindow::editAutomation(const QJsonObject &existing, const QJsonObject &workflow)
{
    if (!online_) {
        showStatus(tr("Connect the runtime to manage automations."), true);
        return;
    }
    QDialog dialog(this);
    dialog.setWindowTitle(existing.isEmpty() ? tr("Add Automation") : tr("Edit automation"));
    dialog.resize(650, 500);
    auto *form = new QFormLayout(&dialog);
    auto *name = new QLineEdit(existing.value("name").toString(workflow.value("title").toString()));
    name->setMaxLength(100);
    auto *prompt = new QPlainTextEdit(existing.value("prompt").toString(workflow.value("prompt").toString()));
    prompt->setFixedHeight(86);
    auto *binding = new QComboBox;
    binding->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
    binding->setMinimumContentsLength(18);
    binding->addItem(tr("Choose a repository binding"), QString());
    for (const auto &value : repositories_) {
        const auto row = value.toObject();
        const auto id = row.value("binding_id").toString();
        if (row.value("status").toString() != "ARCHIVED" && !id.isEmpty())
            binding->addItem(row.value("display_name").toString() + " · " + row.value("branch").toString(), id);
    }
    binding->setCurrentIndex(qMax(0, binding->findData(existing.value("binding_id").toString())));
    auto *kind = new QComboBox;
    kind->addItem(tr("Manual"), QStringLiteral("MANUAL"));
    kind->addItem(tr("Daily schedule"), QStringLiteral("DAILY"));
    const auto oldTrigger = existing.value("trigger").toObject();
    kind->setCurrentIndex(oldTrigger.value("kind").toString() == "DAILY" ? 1 : 0);
    auto *time = new QTimeEdit(QTime(oldTrigger.value("hour").toInt(9), oldTrigger.value("minute").toInt()));
    time->setDisplayFormat(QStringLiteral("HH:mm"));
    auto *timezone = new QComboBox;
    timezone->setEditable(true);
    timezone->setInsertPolicy(QComboBox::NoInsert);
    QStringList zones;
    for (const auto &zone : QTimeZone::availableTimeZoneIds())
        zones.append(QString::fromUtf8(zone));
    if (!zones.contains("UTC"))
        zones.prepend("UTC");
    timezone->addItems(zones);
    timezone->setCurrentText(oldTrigger.value("timezone").toString(QString::fromUtf8(QTimeZone::systemTimeZoneId())));
    auto *enabled = new QCheckBox(tr("Enable future occurrences"));
    enabled->setChecked(existing.value("enabled").toBool(false));
    auto *preview = new QPushButton(tr("Preview schedule"));
    auto *resultLabel = textLabel(QString(), "meta");
    resultLabel->setMinimumHeight(29);
    auto *limits = textLabel(
        tr("Profile: local handoff. Runs stop at approval when execution is unavailable. This definition grants no write or release access."),
        "meta");
    form->addRow(tr("Name"), name);
    form->addRow(tr("Repository"), binding);
    form->addRow(tr("Prompt"), prompt);
    form->addRow(tr("Trigger"), kind);
    form->addRow(tr("Local time"), time);
    form->addRow(tr("IANA time zone"), timezone);
    form->addRow(enabled);
    form->addRow(limits);
    form->addRow(preview);
    form->addRow(resultLabel);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Save | QDialogButtonBox::Cancel);
    form->addRow(buttons);

    const auto trigger = [kind, time, timezone] {
        QJsonObject selected{{"kind", kind->currentData().toString()}};
        if (kind->currentData().toString() == "DAILY") {
            selected.insert("hour", time->time().hour());
            selected.insert("minute", time->time().minute());
            selected.insert("timezone", timezone->currentText().trimmed());
        }
        return selected;
    };
    const auto checked = std::make_shared<QJsonObject>();
    auto *save = buttons->button(QDialogButtonBox::Save);
    const auto enableSave = [=] {
        const auto selected = trigger();
        const bool daily = selected.value("kind").toString() == "DAILY";
        time->setEnabled(daily);
        timezone->setEnabled(daily);
        preview->setEnabled(daily);
        save->setEnabled((!daily || *checked == selected) && !name->text().trimmed().isEmpty() &&
                         !binding->currentData().toString().isEmpty() && !prompt->toPlainText().trimmed().isEmpty());
    };
    enableSave();
    connect(kind, &QComboBox::currentIndexChanged, &dialog, enableSave);
    connect(time, &QTimeEdit::timeChanged, &dialog, enableSave);
    connect(timezone, &QComboBox::currentTextChanged, &dialog, enableSave);
    connect(name, &QLineEdit::textChanged, &dialog, enableSave);
    connect(binding, &QComboBox::currentIndexChanged, &dialog, enableSave);
    connect(prompt, &QPlainTextEdit::textChanged, &dialog, enableSave);
    connect(preview, &QPushButton::clicked, &dialog, [this, &dialog, resultLabel, trigger, checked, enableSave] {
        const QPointer<QDialog> guard(&dialog);
        const auto selection = trigger();
        call("automation.preview", {{"trigger", selection}},
             [this, guard, resultLabel, selection, checked, enableSave](const QJsonObject &result, const QString &error) {
            if (!guard)
                return;
            if (!error.isEmpty()) {
                resultLabel->setText(error);
                return;
            }
            *checked = selection;
            QStringList dates;
            for (const auto &entry : result.value("occurrences").toArray()) {
                const auto date = when(entry);
                if (!date.isEmpty())
                    dates.append(date);
            }
            if (dates.isEmpty())
                dates.append(when(result.value("next_at")));
            resultLabel->setText(tr("Upcoming: %1").arg(dates.join(QStringLiteral(" · "))));
            enableSave();
        });
    });
    connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    if (dialog.exec() != QDialog::Accepted)
        return;
    QJsonObject params{{"name", name->text().trimmed()}, {"binding_id", binding->currentData().toString()},
        {"prompt", prompt->toPlainText().trimmed()}, {"profile", "handoff"},
        {"trigger", trigger()}, {"enabled", enabled->isChecked()}};
    const auto templateId = workflow.value("id").toString(existing.value("template_id").toString());
    if (!templateId.isEmpty())
        params.insert("template_id", templateId);
    if (existing.isEmpty()) {
        params.insert("key", idempotencyKey());
    } else {
        params.insert("id", existing.value("id"));
        params.insert("revision", existing.value("revision"));
    }
    call(existing.isEmpty() ? "automation.create" : "automation.update", params,
         [this](const QJsonObject &, const QString &error) {
        if (!error.isEmpty())
            showError(error);
        else {
            showStatus(tr("Automation definition saved."));
            refreshAutomations();
        }
    });
}

void MainWindow::runAutomation(const QJsonObject &automation)
{
    if (!online_) {
        showStatus(tr("Connect the runtime to manage automations."), true);
        QTimer::singleShot(0, this, &MainWindow::renderAutomationCards);
        return;
    }
    const auto id = automation.value("id").toString();
    const bool inactive = !automation.value("enabled").toBool();
    if (inactive && QMessageBox::question(this, tr("Run inactive automation"),
            tr("Queue one run without enabling the schedule?"), QMessageBox::Yes | QMessageBox::Cancel,
            QMessageBox::Cancel) != QMessageBox::Yes) {
        QTimer::singleShot(0, this, &MainWindow::renderAutomationCards);
        return;
    }
    auto key = runKeys_.value(id);
    if (key.isEmpty()) {
        key = idempotencyKey();
        runKeys_.insert(id, key);
    }
    call("automation.run", {{"id", id}, {"key", key}, {"confirm_inactive", inactive}},
         [this, id](const QJsonObject &run, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            renderAutomationCards();
            return;
        }
        runKeys_.remove(id);
        showStatus(tr("Run %1 · %2").arg(run.value("id").toString(), run.value("status").toString()));
        refreshAutomations();
    });
}

void MainWindow::toggleAutomation(const QJsonObject &automation)
{
    if (!online_)
        return;
    const auto enabled = automation.value("enabled").toBool();
    call("automation.pause", {{"id", automation.value("id")}, {"revision", automation.value("revision")},
                              {"enabled", !enabled}}, [this](const QJsonObject &, const QString &error) {
        if (!error.isEmpty())
            showError(error);
        else
            refreshAutomations();
    });
}

void MainWindow::automationHistory(const QJsonObject &automation)
{
    call("automation.runs", {{"id", automation.value("id")}, {"limit", 30}},
         [this, automation](const QJsonObject &result, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        QDialog dialog(this);
        dialog.setWindowTitle(tr("Run history · %1").arg(automation.value("name").toString()));
        dialog.resize(730, 370);
        auto *layout = new QVBoxLayout(&dialog);
        auto *table = new QTableWidget;
        table->setColumnCount(4);
        table->setHorizontalHeaderLabels({tr("Run"), tr("State"), tr("Occurrence"), tr("Outcome")});
        table->horizontalHeader()->setStretchLastSection(true);
        table->setSelectionBehavior(QAbstractItemView::SelectRows);
        table->setSelectionMode(QAbstractItemView::SingleSelection);
        table->setEditTriggers(QAbstractItemView::NoEditTriggers);
        const auto runs = result.value("items").toArray();
        table->setRowCount(int(runs.size()));
        for (qsizetype index = 0; index < runs.size(); ++index) {
            const auto row = runs[index].toObject();
            const QStringList fields{row.value("id").toString(), row.value("status").toString(),
                when(row.value("occurrence_at")), row.value("outcome").toString()};
            for (qsizetype column = 0; column < fields.size(); ++column) {
                auto *cell = new QTableWidgetItem(fields[column]);
                cell->setToolTip(fields[column].toHtmlEscaped());
                table->setItem(int(index), int(column), cell);
            }
        }
        layout->addWidget(table, 1);
        auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close);
        auto *cancel = buttons->addButton(tr("Cancel pending run"), QDialogButtonBox::ActionRole);
        cancel->setEnabled(false);
        connect(table, &QTableWidget::itemSelectionChanged, &dialog, [table, cancel, runs] {
            const auto index = table->currentRow();
            const auto status = index >= 0 ? runs[index].toObject().value("status").toString() : QString();
            cancel->setEnabled(status == "QUEUED" || status == "WAITING_APPROVAL" || status == "BLOCKED");
        });
        connect(cancel, &QPushButton::clicked, &dialog, [this, table, runs, &dialog] {
            const auto index = table->currentRow();
            if (index < 0)
                return;
            call("automation.cancel", {{"run_id", runs[index].toObject().value("id")}},
                 [this](const QJsonObject &value, const QString &cancelError) {
                if (!cancelError.isEmpty())
                    showError(cancelError);
                else {
                    showStatus(tr("Run status · %1").arg(value.value("status").toString()));
                    refreshAutomations();
                }
            });
            dialog.accept();
        });
        layout->addWidget(buttons);
        connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        dialog.exec();
    });
}
