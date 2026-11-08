#pragma once

#include <QCoreApplication>
#include <QJsonArray>
#include <QSet>
#include <QWidget>

class QLabel;
class QLineEdit;
class QListWidget;
class QPushButton;

class RepositoryPicker final : public QWidget {
    Q_DECLARE_TR_FUNCTIONS(RepositoryPicker)
public:
    explicit RepositoryPicker(bool multiple, const QString &selected = {}, QWidget *parent = nullptr);
    QJsonArray selection() const;
    void populate(const QJsonArray &catalog, bool loaded);
    QPushButton *manageButton() const { return manage_; }
    QListWidget *list() const { return list_; }

private:
    void filter();

    QLineEdit *search_;
    QListWidget *list_;
    QPushButton *manage_;
    QLabel *status_;
    QString preselected_;
    bool loaded_ = false;
};
