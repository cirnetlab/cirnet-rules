#pragma once

#include "runtime_client.h"

#include <QHash>
#include <QJsonArray>
#include <QMainWindow>
#include <QPointer>
#include <QSet>
#include <QStringList>
#include <QVector>

class QAction;
class QButtonGroup;
class QCheckBox;
class QCloseEvent;
class QComboBox;
class QFrame;
class QHBoxLayout;
class QLabel;
class QLineEdit;
class QListWidget;
class QPlainTextEdit;
class QPushButton;
class QScrollArea;
class QSortFilterProxyModel;
class QSpinBox;
class QSplitter;
class QStackedWidget;
class QStandardItemModel;
class QTableView;
class QTimer;
class QToolButton;
class QVBoxLayout;
class NotificationCenter;
class ThemeManager;
class AppearanceColorField;
class GovernancePanel;

#include "theme.h"

class MainWindow final : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(RuntimeClient &client, QWidget *parent = nullptr,
                        bool readyOnConstruction = false, const QString &startupError = {},
                        bool followSavedLanguage = false);

signals:
    void localRuntimeReady();

protected:
    void closeEvent(QCloseEvent *event) override;

private:
    void setConnection(bool ready, const QString &status);
    void showPage(QWidget *page);
    void updateSidebarLayout();
    void updateConversationSelection();
    void navigateConversation(const QString &id);
    void showSettings(const QString &section = {});
    void closeSettings();
    QString settingsSection() const;
    void selectSettingsSection(const QString &section);
    QWidget *makeSettings();
    QWidget *makeAppearanceSettings();
    void updateRuntimeDetails();
    void reconnectRuntime();
    void changeSidebar(bool collapsed);
    void updateShellIcons();
    void loadPreferences();
    void schedulePreferencesSave();
    void savePreferences();
    void applyPreferences(const QJsonObject &settings);
    void applyAppearanceControls(const QJsonObject &settings);
    void applyAppearance();
    QJsonObject appearanceSettings() const;
    QJsonObject preferenceSettings() const;
    void stopRuntimeAndClose();
    void call(const QString &method, QJsonObject params, RuntimeClient::Reply reply);
    void readLatest(RuntimeClient::ViewRead purpose, QJsonObject params, RuntimeClient::Reply reply);
    void showStatus(const QString &text, bool error = false);
    void showError(const QString &text);

    QWidget *makeRepositories();
    void refreshRepositories();
    void appendRepositories();
    void fetchRepositoryPage();
    void renderRepositoryRows();
    void applyRepositoryFilter();
    void restoreRepositoryScroll();
    void ensureRepositoryCatalog(std::function<void(bool)> done);
    QJsonObject repositoryLayout() const;
    void restoreRepositoryLayout(const QJsonObject &layout);
    void checkRepository(bool openFolder);
    void updateRepositoryActions();
    QJsonObject repositorySelection() const;
    void chooseRepository(bool chooseFile, const QJsonObject &existing = {});
    void previewRepository(const QString &path, const QJsonObject &existing = {});
    void confirmRepository(const QString &path, const QJsonObject &existing, const QJsonObject &preview);
    void renameRepository();
    void archiveRepository();
    void selectRepository();

    QWidget *makeConversations();
    QWidget *makeConversationHistory();
    enum class TranscriptPage { Latest, Newer, Older };
    bool canLoadMoreConversations() const;
    void refreshConversations(bool append = false);
    void newConversation(const QString &preselected = {});
    void openConversation(const QString &id, TranscriptPage page = TranscriptPage::Latest,
                          quint64 acknowledgedEvents = 0);
    void suspendConversationReads();
    void editConversation(bool archive, const QJsonObject &record = {});
    void deleteConversationData(const QString &id, const QString &messageId = {}, std::function<void(bool)> onComplete = {});
    void forgetConversation(const QString &id, bool deleted);
    void applyTranscriptPage(const QJsonObject &data, TranscriptPage page);
    void clearCurrentDraft();
    void restoreCurrentDraft();
    void updateConversationSummary();
    void updateConversationAvailability();
    void sendMessage();
    void refreshConversationEvents();
    void conversationTaskDetails();
    void cancelConversationTask();

    QWidget *makeAutomations();
    void refreshAutomations(bool append = false);
    void renderAutomationCards();
    void editAutomation(const QJsonObject &existing = {});
    void chooseAutomationFilter();
    void updateAutomationRepositoryFilter();
    void updateAutomationAvailability();
    void runAutomation(const QJsonObject &automation);
    void toggleAutomation(const QJsonObject &automation);
    void deleteAutomation(const QJsonObject &automation);
    void forgetAutomations(const QStringList &ids);
    void showAutomationBlockers(const QJsonObject &automation, const QJsonObject &result, const QString &error);
    void automationHistory(const QJsonObject &automation);

    RuntimeClient &client_;
    bool online_ = false;
    bool connecting_ = true;
    bool shutdownPending_ = false;
    bool closeAfterSave_ = false;
    QString connectionStatus_;
    ThemeManager *theme_ = nullptr;
    NotificationCenter *notifications_ = nullptr;
    QSplitter *splitter_ = nullptr;
    QFrame *sidebar_ = nullptr;
    QVBoxLayout *sidebarRail_ = nullptr;
    QLabel *brand_ = nullptr;
    QHBoxLayout *identityLayout_ = nullptr;
    QHBoxLayout *localLayout_ = nullptr;
    QToolButton *sidebarToggle_ = nullptr;
    QToolButton *settingsButton_ = nullptr;
    QToolButton *localStatus_ = nullptr;
    QPushButton *chatButton_ = nullptr;
    QPushButton *automationsButton_ = nullptr;
    QWidget *historyPage_ = nullptr;
    QAction *newChatAction_ = nullptr;
    int sidebarWidth_ = 260;
    bool sidebarCollapsed_ = false;
    bool applyingPreferences_ = false;
    bool preferencesLoaded_ = false;
    bool preferencesLoading_ = false;
    bool preferencesDirty_ = false;
    bool preferencesSaving_ = false;
    bool preferencesRetryRequired_ = false;
    qint64 preferencesRevision_ = 0;
    QJsonObject savedPreferences_;
    QTimer *preferencesTimer_ = nullptr;
    QButtonGroup *themeChoice_ = nullptr;
    QToolButton *appearanceAdvancedToggle_ = nullptr;
    QPushButton *appearanceReset_ = nullptr;
    QCheckBox *reducedMotion_ = nullptr;
    QSpinBox *densityChoice_ = nullptr;
    QSpinBox *textScale_ = nullptr;
    QComboBox *accentChoice_ = nullptr;
    QComboBox *uiFontChoice_ = nullptr;
    QSpinBox *contentScaleChoice_ = nullptr;
    QSpinBox *codeScaleChoice_ = nullptr;
    QComboBox *languageChoice_ = nullptr;
    bool followSavedLanguage_ = false;
    AppearanceColorField *backgroundColor_ = nullptr;
    AppearanceColorField *foregroundColor_ = nullptr;
    AppearanceColorField *accentColor_ = nullptr;
    QComboBox *notificationChoice_ = nullptr;
    QLabel *preferencesStatus_ = nullptr;
    QPushButton *preferencesRetry_ = nullptr;
    QLabel *runtimeStatus_ = nullptr;
    QLabel *runtimeCapabilities_ = nullptr;
    QLabel *aboutDetails_ = nullptr;
    QPushButton *reconnect_ = nullptr;
    QStackedWidget *pages_ = nullptr;
    QWidget *settingsPage_ = nullptr;
    GovernancePanel *governance_ = nullptr;
    QWidget *previousPage_ = nullptr;
    QListWidget *settingsNavigation_ = nullptr;
    QLineEdit *settingsSearch_ = nullptr;
    QPointer<QWidget> settingsReturnFocus_;
    QStackedWidget *settingsPages_ = nullptr;
    QWidget *repositoriesPage_ = nullptr;
    QWidget *conversationsPage_ = nullptr;
    QWidget *automationsPage_ = nullptr;
    QJsonObject runtimeInfo_;

    QTableView *repositoryTable_ = nullptr;
    QStandardItemModel *repositoryModel_ = nullptr;
    QSortFilterProxyModel *repositoryProxy_ = nullptr;
    QLineEdit *repoSearch_ = nullptr;
    QComboBox *repoStateFilter_ = nullptr;
    QCheckBox *repoArchived_ = nullptr;
    QLabel *repoSummary_ = nullptr;
    QLabel *repoDetails_ = nullptr;
    QAction *repoRelink_ = nullptr;
    QAction *repoRename_ = nullptr;
    QAction *repoArchive_ = nullptr;
    QAction *repoCheck_ = nullptr;
    QAction *repoOpen_ = nullptr;
    QAction *repoCopy_ = nullptr;
    QPushButton *repoSelect_ = nullptr;
    QPushButton *repoMore_ = nullptr;
    QJsonArray repositories_;
    QJsonValue repoCursor_;
    QJsonValue repoIncomingCursor_;
    bool repoLoading_ = false;
    bool repoComplete_ = false;
    bool repoRestoring_ = false;
    bool repoActionPending_ = false;
    int repoRefreshRows_ = 0;
    QJsonArray repoIncoming_;
    QVector<std::function<void(bool)>> repoWaiters_;
    QString repositorySelectedId_;
    int repositoryScroll_ = 0;
    int repositoryHScroll_ = 0;
    QHash<QString, QJsonObject> repositoryHealth_;
    quint64 repoRequest_ = 0;

    QLineEdit *conversationSearch_ = nullptr;
    QListWidget *conversationList_ = nullptr;
    QPushButton *conversationActions_ = nullptr;
    QPushButton *cancelTask_ = nullptr;
    QPushButton *send_ = nullptr;
    QPushButton *olderMessages_ = nullptr;
    QPushButton *latestMessages_ = nullptr;
    QLabel *conversationTitle_ = nullptr;
    QLabel *conversationDetail_ = nullptr;
    QPlainTextEdit *transcript_ = nullptr;
    QPlainTextEdit *composer_ = nullptr;
    QJsonObject conversation_;
    QJsonArray conversationTasks_;
    QJsonArray conversationBindings_;
    QJsonArray conversationBlockers_;
    struct TranscriptMessage {
        QString id, role, content, displayed;
        qint64 ordinal = 0;
        qsizetype blocks = 0;
        bool shortened = false;

        bool sameIdentity(const TranscriptMessage &other) const;
        qsizetype retainedBytes() const;
        void format(const QString &you, const QString &abbreviated);
    };
    QList<TranscriptMessage> transcriptRows_;
    QJsonValue transcriptBefore_;
    qint64 transcriptNewest_ = 0;
    bool transcriptDetached_ = false;
    QJsonValue conversationCursor_;
    bool conversationLoading_ = false;
    bool conversationListInFlight_ = false;
    bool conversationRefreshQueued_ = false;
    QString currentConversation_;
    QHash<QString, QString> conversationDrafts_, conversationRetryKeys_, conversationRetryContents_;
    QHash<QString, QString> clearedDrafts_;
    QSet<QString> conversationDeletes_;
    QSet<QString> conversationSending_;
    quint64 conversationRequest_ = 0;
    quint64 navigationRevision_ = 0;
    struct ConversationReads {
        quint64 visit = 0;
        quint64 detail = 0;
        quint64 cursor = 0;
        bool loading = false;
        bool polling = false;
        bool newerQueued = false;
        bool refreshOnReturn = false;
        bool confirmed = false;
    } conversationReads_;

    QLineEdit *automationSearch_ = nullptr;
    QComboBox *automationFilter_ = nullptr;
    QPushButton *automationRepoFilter_ = nullptr;
    QString automationBindingFilter_;
    QVBoxLayout *automationCards_ = nullptr;
    QScrollArea *automationScroll_ = nullptr;
    QLabel *automationState_ = nullptr;
    QPushButton *automationMore_ = nullptr;
    QJsonArray automations_;
    QHash<QString, QFrame *> automationCardWidgets_;
    QSet<QString> automationRunsPending_;
    QSet<QString> automationDeletesPending_;
    QJsonValue automationCursor_;
    quint64 automationRequest_ = 0;
    int cardColumns_ = 0;
    QHash<QString, QString> runKeys_;
    QTimer *poll_ = nullptr;
};
