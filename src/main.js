const { invoke } = window.__TAURI__.core;

const input = document.getElementById("reminder-input");
const addButton = document.getElementById("add-button");
const reminderList = document.getElementById("reminder-list");
const emptyMessage = document.getElementById("empty-message");


function renderReminders(reminders) {
    reminderList.innerHTML = "";

    if (reminders.length === 0) {
        emptyMessage.style.display = "block";
        return;
    }

    emptyMessage.style.display = "none";

    reminders.forEach((reminder) => {
        const listItem = document.createElement("li");

        listItem.className = "reminder";

        // Checkbox
        const checkbox = document.createElement("input");

        checkbox.type = "checkbox";
        checkbox.checked = reminder.completed;

        checkbox.addEventListener("change", () => {
            toggleReminder(reminder.id);
        });


        // Reminder text
        const text = document.createElement("span");

        text.textContent = reminder.text;

        if (reminder.completed) {
            text.style.textDecoration = "line-through";
            text.style.color = "#888";
        }


        // Edit button
        const editButton = document.createElement("button");

        editButton.textContent = "Edit";

        editButton.addEventListener("click", () => {
            editReminder(reminder);
        });


        // Delete button
        const deleteButton = document.createElement("button");

        deleteButton.textContent = "Delete";
        deleteButton.className = "delete-button";

        deleteButton.addEventListener("click", () => {
            deleteReminder(reminder.id);
        });


        listItem.appendChild(checkbox);
        listItem.appendChild(text);
        listItem.appendChild(editButton);
        listItem.appendChild(deleteButton);

        reminderList.appendChild(listItem);
    });
}




async function loadReminders() {

    try {

        const reminders = await invoke("get_reminders");

        renderReminders(reminders);

    } catch (error) {

        console.error("Failed to load reminders:", error);

    }
}


async function addReminder() {

    const text = input.value.trim();

    if (text === "") {
        return;
    }

    try {

        await invoke("add_reminder", {
            text: text
        });

        input.value = "";

        await loadReminders();

    } catch (error) {

        console.error("Failed to add reminder:", error);

    }
}


async function deleteReminder(id) {

    try {

        await invoke("delete_reminder", {
            id: id
        });

        await loadReminders();

    } catch (error) {

        console.error("Failed to delete reminder:", error);

    }
}

async function editReminder(reminder) {
    const newText = prompt(
        "Edit reminder:",
        reminder.text
    );

    if (newText === null) {
        return;
    }

    const trimmedText = newText.trim();

    if (trimmedText === "") {
        return;
    }

    try {
        await invoke("update_reminder", {
            id: reminder.id,
            text: trimmedText
        });

        await loadReminders();

    } catch (error) {
        console.error(
            "Failed to update reminder:",
            error
        );
    }
}


addButton.addEventListener("click", addReminder);


input.addEventListener("keydown", (event) => {

    if (event.key === "Enter") {
        addReminder();
    }

});


loadReminders();
