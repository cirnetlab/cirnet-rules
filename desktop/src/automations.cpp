#include "main_window.h"
#include "compact_text_edit.h"
#include "repository_picker.h"

#include <QCheckBox>
#include <QComboBox>
#include <QDateTime>
#include <QDialog>
#include <QDialogButtonBox>
#include <QEvent>
#include <QFormLayout>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QLocale>
#include <QMenu>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QPointer>
#include <QPushButton>
#include <QScrollArea>
#include <QScreen>
#include <QStackedWidget>
#include <QTableWidget>
#include <QTimeEdit>
#include <QTimeZone>
#include <QTimer>
#include <QUuid>
#include <QVBoxLayout>
#include <functional>
#include <memory>
#include <utility>

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

bool morePages(const QJsonValue &cursor)
{
    return !cursor.isNull() && !cursor.isUndefined();
}

bool updateDefinition(QJsonArray &rows, const QJsonObject &definition, const QString &query,
                      const QString &status, const QString &binding)
{
    const auto id = definition.value("id").toString();
    if (id.isEmpty())
        return false;
    const bool matches = definition.value("name").toString().contains(query, Qt::CaseInsensitive)
        && (status.isEmpty() || definition.value("enabled").toBool() == (status == "active"))
        && (binding.isEmpty() || definition.value("binding_id").toString() == binding);
    for (qsizetype index = 0; index < rows.size(); ++index) {
        if (rows[index].toObject().value("id").toString() != id)
            continue;
        if (matches)
            rows[index] = definition;
        else
            rows.removeAt(index);
        return true;
    }
    return false;
}

void reflowFilters(QScrollArea *scroll)
{
    auto *bar = scroll->parentWidget()->findChild<QWidget *>("automationFilters");
    auto *grid = qobject_cast<QGridLayout *>(bar->layout());
    auto *search = bar->findChild<QLineEdit *>("automationSearch");
    auto *status = bar->findChild<QComboBox *>();
    auto *repository = bar->findChild<QPushButton *>("automationRepoFilter");
    auto *refresh = bar->findChild<QPushButton *>("refreshAutomations");
    const bool wide = scroll->viewport()->width() >= 110 + status->sizeHint().width()
        + repository->sizeHint().width() + refresh->sizeHint().width() + 3 * grid->horizontalSpacing();
    if (bar->property("wide").toBool() == wide)
        return;
    bar->setProperty("wide", wide);
    for (auto *widget : QList<QWidget *>{search, status, repository, refresh})
        grid->removeWidget(widget);
    grid->addWidget(search, 0, 0, 1, wide ? 1 : 3);
    grid->addWidget(status, wide ? 0 : 1, wide ? 1 : 0);
    grid->addWidget(repository, wide ? 0 : 1, wide ? 2 : 1);
    grid->addWidget(refresh, wide ? 0 : 1, wide ? 3 : 2);
}

class AutomationScroll final : public QScrollArea {
public:
    explicit AutomationScroll(std::function<void()> onResize) : onResize_(std::move(onResize)) {}

protected:
    bool viewportEvent(QEvent *event) override
    {
        const auto result = QScrollArea::viewportEvent(event);
        if (event->type() == QEvent::Resize && !reflowPending_) {
            reflowPending_ = true;
            QTimer::singleShot(0, this, [this] {
                reflowPending_ = false;
                onResize_();
            });
        }
        return result;
    }

private:
    std::function<void()> onResize_;
    bool reflowPending_ = false;
};

class AutomationCard final : public QFrame {
    Q_DECLARE_TR_FUNCTIONS(AutomationCard)
public:
    using Action = std::function<void(const QJsonObject &)>;
    AutomationCard(Action runAction, Action editAction, Action toggleAction, Action historyAction)
    {
        setObjectName("card");
        setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
        auto *layout = new QVBoxLayout(this);
        layout->setContentsMargins(12, 10, 12, 10);
        layout->setSpacing(5);
        auto *header = new QHBoxLayout;
        header->setSpacing(5);
        title_ = textLabel({}, "automationCardTitle");
        title_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        header->addWidget(title_, 1);
        run_ = new QPushButton(tr("Run now"));
        run_->setObjectName("automationRun");
        header->addWidget(run_);
        auto *more = new QPushButton(tr("Actions"));
        more->setObjectName("automationActions");
        header->addWidget(more);
        layout->addLayout(header);
        body_ = textLabel({}, "cardBody");
        body_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        layout->addWidget(body_);
        tags_ = textLabel({}, "meta");
        tags_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        layout->addWidget(tags_);
        state_ = textLabel({}, "meta");
        state_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        layout->addWidget(state_);
        auto *menu = new QMenu(more);
        edit_ = menu->addAction(tr("Edit definition"));
        edit_->setObjectName("editAutomationDefinition");
        toggle_ = menu->addAction(tr("Pause"));
        auto *history = menu->addAction(tr("Run history"));
        history->setObjectName("showAutomationHistory");
        more->setMenu(menu);
        connect(run_, &QPushButton::clicked, this, [this, action = std::move(runAction)] { action(record_); });
        connect(edit_, &QAction::triggered, this, [this, action = std::move(editAction)] { action(record_); });
        connect(toggle_, &QAction::triggered, this, [this, action = std::move(toggleAction)] { action(record_); });
        connect(history, &QAction::triggered, this, [this, action = std::move(historyAction)] { action(record_); });
    }

    void update(const QJsonObject &item, const QString &repository, bool online, bool pending)
    {
        run_->setEnabled(online && !pending);
        edit_->setEnabled(online && !pending);
        toggle_->setEnabled(online && !pending);
        if (record_ == item && repository_ == repository)
            return;
        record_ = item;
        repository_ = repository;
        const auto name = item.value("name").toString();
        title_->setText(name);
        title_->setToolTip(name.toHtmlEscaped());
        run_->setAccessibleName(tr("Run %1 now").arg(name));
        const auto prompt = item.value("prompt").toString().simplified();
        body_->setText(prompt.size() > 140 ? prompt.left(137) + QStringLiteral("…") : prompt);
        const auto trigger = item.value("trigger").toObject();
        QString time = tr("Manual");
        if (trigger.value("kind").toString() == "DAILY") {
            const QTime slot(trigger.value("hour").toInt(), trigger.value("minute").toInt());
            time = tr("Daily %1 · %2").arg(slot.toString("HH:mm"), trigger.value("timezone").toString());
        }
        const auto next = when(item.value("next_at"));
        QStringList tags{repository, time};
        if (!next.isEmpty())
            tags.append(tr("Next %1").arg(next));
        tags.removeAll(QString());
        tags_->setText(tags.join(QStringLiteral(" · ")));
        const auto last = item.value("last_run").toObject();
        state_->setText(last.isEmpty() ? tr("No runs yet") : tr("Last run: %1").arg(last.value("status").toString()));
        toggle_->setText(item.value("enabled").toBool() ? tr("Pause") : tr("Enable"));
    }

private:
    QJsonObject record_;
    QString repository_;
    QLabel *title_ = nullptr;
    QLabel *body_ = nullptr;
    QLabel *tags_ = nullptr;
    QLabel *state_ = nullptr;
    QPushButton *run_ = nullptr;
    QAction *edit_ = nullptr;
    QAction *toggle_ = nullptr;
};
}

QWidget *MainWindow::makeAutomations()
{
    auto *page = new QWidget;
    page->setObjectName("automationsPage");
    auto *outer = new QHBoxLayout(page);
    outer->setContentsMargins(24, 24, 24, 12);
    auto *panel = new QWidget;
    panel->setMaximumWidth(1050);
    auto *layout = new QVBoxLayout(panel);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(12);
    outer->addStretch();
    outer->addWidget(panel, 100);
    outer->addStretch();
    panel->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);

    auto *top = new QHBoxLayout;
    top->addWidget(textLabel(tr("Automations"), "heading"), 1);
    auto *add = new QPushButton(tr("Add Automation"));
    add->setObjectName("addAutomation");
    add->setProperty("primary", true);
    add->setAccessibleName(tr("Add Automation"));
    top->addWidget(add);
    layout->addLayout(top);
    layout->addWidget(textLabel(tr("Manage local schedules and inspect run history."), "lead"));

    auto *filterBar = new QWidget;
    filterBar->setObjectName("automationFilters");
    auto *filter = new QGridLayout(filterBar);
    filter->setContentsMargins(0, 0, 0, 0);
    filter->setColumnStretch(0, 1);
    automationSearch_ = new QLineEdit;
    automationSearch_->setObjectName("automationSearch");
    automationSearch_->setPlaceholderText(tr("Search automations…"));
    automationSearch_->setAccessibleName(tr("Search automations"));
    automationSearch_->setClearButtonEnabled(true);
    automationFilter_ = new QComboBox;
    automationFilter_->setAccessibleName(tr("Filter automation status"));
    automationFilter_->addItem(tr("All statuses"), QString());
    automationFilter_->addItem(tr("Active"), QStringLiteral("active"));
    automationFilter_->addItem(tr("Inactive"), QStringLiteral("inactive"));
    automationRepoFilter_ = new QPushButton(tr("All repositories"));
    automationRepoFilter_->setObjectName("automationRepoFilter");
    automationRepoFilter_->setAccessibleName(tr("Filter automation repository"));
    auto *refresh = new QPushButton(tr("Refresh"));
    refresh->setObjectName("refreshAutomations");
    filter->addWidget(automationSearch_, 0, 0, 1, 3);
    filter->addWidget(automationFilter_, 1, 0);
    filter->addWidget(automationRepoFilter_, 1, 1);
    filter->addWidget(refresh, 1, 2);
    layout->addWidget(filterBar);
    automationState_ = textLabel(tr("Connect to load automations."), "meta");
    automationState_->setObjectName("automationState");
    layout->addWidget(automationState_);

    automationScroll_ = new AutomationScroll([this] {
        reflowFilters(automationScroll_);
        if ((automationScroll_->viewport()->width() >= 710 ? 2 : 1) != cardColumns_)
            renderAutomationCards();
    });
    automationScroll_->setObjectName("automationViewport");
    automationScroll_->setWidgetResizable(true);
    automationScroll_->setFrameShape(QFrame::NoFrame);
    automationScroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    auto *contents = new QWidget;
    automationCards_ = new QVBoxLayout(contents);
    automationCards_->setContentsMargins(0, 2, 4, 0);
    automationCards_->setSpacing(15);
    for (const auto group : {"active", "inactive"}) {
        auto *section = new QWidget(contents);
        section->setObjectName(QString::fromLatin1(group) + QStringLiteral("AutomationSection"));
        auto *stack = new QVBoxLayout(section);
        stack->setContentsMargins(0, 0, 0, 0);
        stack->setSpacing(7);
        auto *heading = textLabel({}, "sectionTitle");
        stack->addWidget(heading);
        auto *grid = new QGridLayout;
        grid->setContentsMargins(0, 0, 0, 0);
        grid->setSpacing(8);
        stack->addLayout(grid);
        automationCards_->addWidget(section);
    }
    automationCards_->addStretch();
    automationScroll_->setWidget(contents);
    layout->addWidget(automationScroll_, 1);
    automationMore_ = new QPushButton(tr("Load more"));
    automationMore_->hide();
    layout->addWidget(automationMore_, 0, Qt::AlignRight);

    auto *searchDelay = new QTimer(page);
    searchDelay->setSingleShot(true);
    searchDelay->setInterval(220);
    connect(automationSearch_, &QLineEdit::textChanged, this, [this, searchDelay] {
        ++automationRequest_;
        automationMore_->setEnabled(false);
        searchDelay->start();
    });
    connect(searchDelay, &QTimer::timeout, this, [this] { refreshAutomations(); });
    connect(automationFilter_, &QComboBox::currentIndexChanged, this, [this] { refreshAutomations(); });
    connect(automationRepoFilter_, &QPushButton::clicked, this, &MainWindow::chooseAutomationFilter);
    connect(automationMore_, &QPushButton::clicked, this, [this] { refreshAutomations(true); });
    connect(refresh, &QPushButton::clicked, this, [this] { refreshAutomations(); });
    connect(add, &QPushButton::clicked, this, [this] { editAutomation(); });
    return page;
}

void MainWindow::updateAutomationAvailability()
{
    if (!automationsPage_)
        return;
    for (const auto name : {"addAutomation", "refreshAutomations", "automationRepoFilter"}) {
        if (auto *button = automationsPage_->findChild<QPushButton *>(QString::fromLatin1(name)))
            button->setEnabled(online_);
    }
    automationFilter_->setEnabled(online_);
    automationSearch_->setEnabled(online_);
    automationMore_->setEnabled(online_);
    if (!online_) {
        automationState_->setText(tr("Connect to load automations."));
        automationState_->show();
    }
    renderAutomationCards();
}

void MainWindow::refreshAutomations(bool append)
{
    if (!online_)
        return;
    QJsonObject params{{"limit", 4}, {"query", automationSearch_->text().trimmed()}};
    const auto status = automationFilter_->currentData().toString();
    if (!status.isEmpty())
        params.insert("enabled", status == "active");
    if (!automationBindingFilter_.isEmpty())
        params.insert("binding_id", automationBindingFilter_);
    if (append && morePages(automationCursor_))
        params.insert("before", automationCursor_);
    const auto request = ++automationRequest_;
    automationMore_->setEnabled(false);
    if (automations_.isEmpty()) {
        automationState_->setText(tr("Loading automations…"));
        automationState_->show();
    }
    call("automation.list", params, [this, request, append](const QJsonObject &result, const QString &error) {
        if (request != automationRequest_)
            return;
        automationMore_->setEnabled(true);
        if (!error.isEmpty()) {
            automationState_->setText(tr("Automations could not load."));
            automationState_->show();
            showStatus(error, true);
            return;
        }
        if (!append)
            automations_ = {};
        for (const auto &row : result.value("items").toArray())
            automations_.append(row);
        automationCursor_ = result.value("next_before");
        automationMore_->setVisible(morePages(automationCursor_));
        const bool filtered = !automationSearch_->text().trimmed().isEmpty()
            || !automationFilter_->currentData().toString().isEmpty() || !automationBindingFilter_.isEmpty();
        automationState_->setText(filtered ? tr("No automations match these filters.")
                                           : tr("No automations yet. Add a local automation to get started."));
        automationState_->setVisible(automations_.isEmpty());
        renderAutomationCards();
    });
}

void MainWindow::renderAutomationCards()
{
    if (!automationScroll_)
        return;
    reflowFilters(automationScroll_);
    const int columns = automationScroll_->viewport()->width() >= 710 ? 2 : 1;
    QHash<QString, QString> repositoryNames;
    for (const auto &value : repositories_) {
        const auto item = value.toObject();
        repositoryNames.insert(item.value("binding_id").toString(), item.value("display_name").toString());
    }
    QSet<QString> remaining;
    for (const auto group : {true, false}) {
        const auto name = QString::fromLatin1(group ? "activeAutomationSection" : "inactiveAutomationSection");
        auto *section = automationScroll_->widget()->findChild<QWidget *>(name);
        if (!section)
            continue;
        auto *title = section->findChild<QLabel *>("sectionTitle");
        auto *grid = section->findChild<QGridLayout *>();
        QList<QWidget *> cards;
        for (const auto &value : automations_) {
            const auto item = value.toObject();
            if (item.value("enabled").toBool() != group)
                continue;
            const auto id = item.value("id").toString();
            if (id.isEmpty())
                continue;
            const auto binding = item.value("binding_id").toString();
            auto repository = repositoryNames.value(binding);
            if (repository.isEmpty() && !binding.isEmpty())
                repository = tr("Saved binding");
            auto *card = static_cast<AutomationCard *>(automationCardWidgets_.value(id));
            if (!card) {
                card = new AutomationCard(
                    [this](const QJsonObject &record) { runAutomation(record); },
                    [this](const QJsonObject &record) { editAutomation(record); },
                    [this](const QJsonObject &record) { toggleAutomation(record); },
                    [this](const QJsonObject &record) { automationHistory(record); });
                card->setParent(section);
                automationCardWidgets_[id] = card;
            }
            card->update(item, repository, online_, automationRunsPending_.contains(id));
            cards.append(card);
            remaining.insert(id);
        }
        const bool unchanged = columns == cardColumns_ && grid->count() == cards.size()
            && [&] {
                for (qsizetype index = 0; index < cards.size(); ++index)
                    if (grid->itemAt(int(index))->widget() != cards[index])
                        return false;
                return true;
            }();
        if (!unchanged) {
            while (auto *layoutItem = grid->takeAt(0))
                delete layoutItem;
            grid->setColumnStretch(0, 1);
            grid->setColumnStretch(1, columns == 2 ? 1 : 0);
            for (qsizetype index = 0; index < cards.size(); ++index) {
                auto *card = cards[index];
                grid->addWidget(card, int(index) / columns, int(index) % columns);
                card->show();
            }
        }
        title->setText((group ? tr("Active · %1 shown") : tr("Inactive · %1 shown")).arg(cards.size()));
        section->setVisible(!cards.isEmpty());
    }
    cardColumns_ = columns;
    for (auto it = automationCardWidgets_.begin(); it != automationCardWidgets_.end();) {
        if (remaining.contains(it.key())) {
            ++it;
        } else {
            it.value()->hide();
            it.value()->deleteLater();
            it = automationCardWidgets_.erase(it);
        }
    }
}

void MainWindow::chooseAutomationFilter()
{
    if (!online_)
        return;
    QDialog dialog(this);
    dialog.setWindowTitle(tr("Filter by repository"));
    dialog.resize(480, 320);
    auto *layout = new QVBoxLayout(&dialog);
    auto *picker = new RepositoryPicker(false, automationBindingFilter_);
    picker->manageButton()->hide();
    layout->addWidget(picker, 1);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
    auto *clear = buttons->addButton(tr("All repositories"), QDialogButtonBox::ResetRole);
    auto *ok = buttons->button(QDialogButtonBox::Ok);
    ok->setText(tr("Apply filter"));
    buttons->button(QDialogButtonBox::Cancel)->setText(tr("Cancel"));
    ok->setEnabled(false);
    connect(picker->list(), &QListWidget::itemSelectionChanged, &dialog, [ok, picker] {
        ok->setEnabled(!picker->selection().isEmpty());
    });
    connect(clear, &QPushButton::clicked, &dialog, [&dialog] { dialog.done(2); });
    connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    const QPointer<QDialog> guard(&dialog);
    ensureRepositoryCatalog([this, guard, picker, ok](bool loaded) {
        if (!guard)
            return;
        picker->populate(repositories_, loaded);
        ok->setEnabled(!picker->selection().isEmpty());
    });
    const auto choice = dialog.exec();
    if (choice == QDialog::Rejected)
        return;
    automationBindingFilter_ = choice == 2 ? QString() : picker->selection().first().toString();
    auto name = tr("All repositories");
    for (const auto &value : repositories_) {
        const auto record = value.toObject();
        if (record.value("binding_id").toString() == automationBindingFilter_) {
            name = record.value("display_name").toString();
            break;
        }
    }
    automationRepoFilter_->setText(automationRepoFilter_->fontMetrics().elidedText(name, Qt::ElideRight, 126));
    automationRepoFilter_->setToolTip(name.toHtmlEscaped());
    refreshAutomations();
}

void MainWindow::editAutomation(const QJsonObject &existing)
{
    if (!online_) {
        showStatus(tr("Connect the runtime to manage automations."), true);
        return;
    }
    if (auto *dialog = findChild<QDialog *>("automationEditor", Qt::FindDirectChildrenOnly)) {
        dialog->show();
        dialog->raise();
        dialog->activateWindow();
        return;
    }
    auto *dialog = new QDialog(this);
    dialog->setObjectName("automationEditor");
    dialog->setAttribute(Qt::WA_DeleteOnClose);
    dialog->setWindowTitle(existing.isEmpty() ? tr("Add Automation") : tr("Edit automation"));
    const auto *screen = window()->screen();
    dialog->resize(screen ? QSize(630, 580).boundedTo(screen->availableGeometry().size() - QSize(32, 72))
                          : QSize(630, 580));
    auto *layout = new QVBoxLayout(dialog);
    auto *scroll = new QScrollArea;
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    auto *content = new QWidget;
    auto *form = new QFormLayout(content);
    form->setContentsMargins(0, 0, 6, 0);
    form->setRowWrapPolicy(QFormLayout::WrapLongRows);
    scroll->setWidget(content);
    layout->addWidget(scroll, 1);
    auto *name = new QLineEdit(existing.value("name").toString());
    name->setObjectName("automationName");
    name->setMaxLength(100);
    auto *prompt = new CompactTextEdit(4);
    prompt->setPlainText(existing.value("prompt").toString());
    prompt->setObjectName("automationPrompt");
    prompt->setAccessibleName(tr("Prompt"));
    auto *binding = new RepositoryPicker(false, existing.value("binding_id").toString());
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
    auto *scheduleStatus = textLabel({}, "meta");
    auto *formStatus = textLabel({}, "meta");
    formStatus->hide();
    auto *limits = textLabel(
        tr("Local handoff. Runs may wait for approval. This definition grants no write or release access."), "meta");
    form->addRow(tr("Name"), name);
    form->addRow(tr("Repository"), binding);
    form->addRow(tr("Prompt"), prompt);
    form->addRow(tr("Trigger"), kind);
    form->addRow(tr("Local time"), time);
    form->addRow(tr("IANA time zone"), timezone);
    form->addRow(enabled);
    form->addRow(preview);
    form->addRow(scheduleStatus);
    form->addRow(limits);
    layout->addWidget(formStatus);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Save | QDialogButtonBox::Cancel);
    layout->addWidget(buttons);

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
    auto *cancel = buttons->button(QDialogButtonBox::Cancel);
    save->setText(tr("Save"));
    cancel->setText(tr("Cancel"));
    const auto enableSave = [=] {
        const auto selected = trigger();
        const bool daily = selected.value("kind").toString() == "DAILY";
        const bool saving = dialog->property("saving").toBool();
        form->setRowVisible(time, daily);
        form->setRowVisible(timezone, daily);
        form->setRowVisible(preview, daily);
        form->setRowVisible(scheduleStatus, daily && !scheduleStatus->text().isEmpty());
        preview->setEnabled(!saving && !dialog->property("previewPending").toBool());
        cancel->setEnabled(!saving);
        binding->manageButton()->setEnabled(!saving);
        save->setEnabled(!saving && (!daily || *checked == selected)
                         && !name->text().trimmed().isEmpty() && !binding->selection().isEmpty()
                         && !prompt->toPlainText().trimmed().isEmpty());
    };
    const QPointer<QDialog> guard(dialog);
    const auto load = [this, guard, binding, enableSave] {
        ensureRepositoryCatalog([this, guard, binding, enableSave](bool loaded) {
            if (!guard)
                return;
            binding->populate(repositories_, loaded);
            enableSave();
        });
    };
    connect(binding->manageButton(), &QPushButton::clicked, dialog, [this, dialog] {
        dialog->setProperty("managingRepositories", true);
        dialog->hide();
        showSettings(QStringLiteral("repositories"));
    });
    connect(pages_, &QStackedWidget::currentChanged, dialog, [this, dialog, load](int) {
        if (pages_->currentWidget() == automationsPage_ && dialog->property("managingRepositories").toBool()) {
            dialog->setProperty("managingRepositories", false);
            dialog->show();
            dialog->raise();
            dialog->activateWindow();
            load();
        }
    });
    enableSave();
    connect(kind, &QComboBox::currentIndexChanged, dialog, enableSave);
    connect(time, &QTimeEdit::timeChanged, dialog, enableSave);
    connect(timezone, &QComboBox::currentTextChanged, dialog, enableSave);
    connect(name, &QLineEdit::textChanged, dialog, enableSave);
    connect(binding->list(), &QListWidget::itemSelectionChanged, dialog, enableSave);
    connect(prompt, &QPlainTextEdit::textChanged, dialog, enableSave);
    connect(preview, &QPushButton::clicked, dialog, [this, guard, scheduleStatus, trigger, checked, enableSave] {
        if (!guard)
            return;
        const auto selection = trigger();
        guard->setProperty("previewPending", true);
        scheduleStatus->setText(tr("Checking upcoming occurrences…"));
        enableSave();
        call("automation.preview", {{"trigger", selection}},
             [this, guard, scheduleStatus, selection, trigger, checked, enableSave](const QJsonObject &result, const QString &error) {
            if (!guard)
                return;
            guard->setProperty("previewPending", false);
            if (selection != trigger()) {
                scheduleStatus->clear();
            } else if (!error.isEmpty()) {
                scheduleStatus->setText(error);
            } else {
                *checked = selection;
                QStringList dates;
                for (const auto &entry : result.value("occurrences").toArray()) {
                    const auto date = when(entry);
                    if (!date.isEmpty())
                        dates.append(date);
                }
                if (dates.isEmpty())
                    dates.append(when(result.value("next_at")));
                scheduleStatus->setText(tr("Upcoming: %1").arg(dates.join(QStringLiteral(" · "))));
            }
            enableSave();
        });
    });
    connect(buttons, &QDialogButtonBox::rejected, dialog, &QDialog::reject);
    connect(dialog, &QDialog::finished, dialog, [dialog] { dialog->setProperty("finished", true); });
    connect(buttons, &QDialogButtonBox::accepted, dialog,
            [this, guard, name, binding, prompt, trigger, enabled, existing, formStatus, enableSave] {
        if (!guard || guard->property("saving").toBool() || binding->selection().isEmpty())
            return;
        if (!online_) {
            formStatus->setText(tr("Reconnect the local runtime to save."));
            formStatus->show();
            return;
        }
        QJsonObject params{{"name", name->text().trimmed()}, {"binding_id", binding->selection().at(0).toString()},
            {"prompt", prompt->toPlainText().trimmed()}, {"profile", "handoff"},
            {"trigger", trigger()}, {"enabled", enabled->isChecked()}};
        const auto templateId = existing.value("template_id").toString();
        if (!templateId.isEmpty())
            params.insert("template_id", templateId);
        if (existing.value("read_paths").isArray())
            params.insert("read_paths", existing.value("read_paths"));
        if (existing.isEmpty()) {
            if (guard->property("submittedPayload").toJsonObject() != params) {
                guard->setProperty("submittedPayload", params);
                guard->setProperty("submissionKey", idempotencyKey());
            }
            params.insert("key", guard->property("submissionKey").toString());
        } else {
            params.insert("id", existing.value("id"));
            params.insert("revision", existing.value("revision"));
        }
        guard->setProperty("saving", true);
        enableSave();
        formStatus->setText(tr("Saving automation…"));
        formStatus->show();
        call(existing.isEmpty() ? "automation.create" : "automation.update", params,
             [this, guard, formStatus, enableSave](const QJsonObject &definition, const QString &error) {
            if (!guard || guard->property("finished").toBool()) {
                if (!error.isEmpty())
                    showStatus(error, true);
                else
                    refreshAutomations();
                return;
            }
            guard->setProperty("saving", false);
            enableSave();
            if (!error.isEmpty()) {
                formStatus->setText(error);
                return;
            }
            guard->accept();
            showStatus(tr("Automation definition saved."));
            if (automationMore_->isEnabled()
                && updateDefinition(automations_, definition, automationSearch_->text().trimmed(),
                                 automationFilter_->currentData().toString(), automationBindingFilter_)
                && !automations_.isEmpty())
                renderAutomationCards();
            else
                refreshAutomations();
        });
    });
    dialog->show();
    name->setFocus();
    load();
}

void MainWindow::runAutomation(const QJsonObject &automation)
{
    if (!online_) {
        showStatus(tr("Connect the runtime to manage automations."), true);
        return;
    }
    const auto id = automation.value("id").toString();
    if (automationRunsPending_.contains(id))
        return;
    automationRunsPending_.insert(id);
    renderAutomationCards();
    const bool inactive = !automation.value("enabled").toBool();
    if (inactive) {
        QMessageBox confirmation(QMessageBox::Question, tr("Run inactive automation"),
            tr("Queue one run without enabling the schedule?"), QMessageBox::Yes | QMessageBox::Cancel, this);
        confirmation.button(QMessageBox::Yes)->setText(tr("Queue run"));
        confirmation.button(QMessageBox::Cancel)->setText(tr("Cancel"));
        confirmation.setDefaultButton(QMessageBox::Cancel);
        if (confirmation.exec() != QMessageBox::Yes) {
            automationRunsPending_.remove(id);
            renderAutomationCards();
            return;
        }
    }
    auto key = runKeys_.value(id);
    if (key.isEmpty()) {
        key = idempotencyKey();
        runKeys_.insert(id, key);
    }
    call("automation.run", {{"id", id}, {"key", key}, {"confirm_inactive", inactive}},
         [this, id](const QJsonObject &run, const QString &error) {
        automationRunsPending_.remove(id);
        if (!error.isEmpty()) {
            showError(error);
            renderAutomationCards();
            return;
        }
        runKeys_.remove(id);
        for (qsizetype index = 0; index < automations_.size(); ++index) {
            auto row = automations_[index].toObject();
            if (row.value("id").toString() != id)
                continue;
            row.insert("last_run", run);
            automations_[index] = row;
            break;
        }
        renderAutomationCards();
        showStatus(tr("Run %1 · %2").arg(run.value("id").toString(), run.value("status").toString()));
        if (!automationMore_->isEnabled())
            refreshAutomations();
    });
}


void MainWindow::toggleAutomation(const QJsonObject &automation)
{
    if (!online_)
        return;
    const auto enabled = automation.value("enabled").toBool();
    call("automation.pause", {{"id", automation.value("id")}, {"revision", automation.value("revision")},
                              {"enabled", !enabled}}, [this, enabled](const QJsonObject &definition, const QString &error) {
        if (!error.isEmpty()) {
            showError(error);
            return;
        }
        showStatus(enabled ? tr("Automation paused.") : tr("Automation enabled."));
        if (automationMore_->isEnabled()
            && updateDefinition(automations_, definition, automationSearch_->text().trimmed(),
                             automationFilter_->currentData().toString(), automationBindingFilter_)
            && !automations_.isEmpty())
            renderAutomationCards();
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
        buttons->button(QDialogButtonBox::Close)->setText(tr("Close"));
        auto *cancel = buttons->addButton(tr("Cancel pending run"), QDialogButtonBox::ActionRole);
        cancel->setEnabled(false);
        connect(table, &QTableWidget::itemSelectionChanged, &dialog, [table, cancel, runs] {
            const auto index = table->currentRow();
            const auto status = index >= 0 ? runs[index].toObject().value("status").toString() : QString();
            cancel->setEnabled(status == "QUEUED" || status == "CLAIMED"
                               || status == "WAITING_APPROVAL" || status == "RECOVERY_REQUIRED");
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
