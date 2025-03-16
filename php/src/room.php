<?php
$room_key = $_GET["key"] ?? null;
$user_id = $_GET["id"] ?? null;
if (is_null($room_key) || preg_match("/^[a-z]{5}$/", $room_key) === 0
    || is_null($user_id) || preg_match("/^[0-9]{1,10}$/", $user_id) === 0 || intval($user_id) < 0 || intval($user_id) > 4294967295
) {
    http_response_code(303);
    header("Location: /");
} elseif (preg_match("/^\/room\.php.*/", $_SERVER['REQUEST_URI']) === 1) {
    http_response_code(303);
    header("Location: /room/" . $room_key . "/" . $user_id);
}
$user_id = intval($user_id);
?>

<!DOCTYPE html>
<html>


<head>
    <title>Room <?php echo $room_key; ?></title>
    <style>
        hr {
            width: 100%;
        }
        .top {
            position: sticky;
            top: 0;
            background: white;
        }
        #user-name {
            font-style: italic;
        }
        #user-name-modifying {
            display: none;
        }
        #close-frame {
            color: orange;
            display: none;
        }
        #error-frame {
            color: red;
            display: none;
        }
    </style>
</head>

<body>
    <div class="top">
        <a href="/"><p>&lt; go home</p></a>
        <h1>Room "<?php echo $room_key; ?>"</h1>
        <div>
            <span id="user-name-frame"><span id="user-name">unknown</span><a href="" id="update-user-name">🖉</a></span>
            <form id="user-name-modifying">
                <input type="text" name="user-name-modifying-input" id="user-name-modifying-input" required>
                <input type="reset" id="user-name-modifying-reset" value="Cancel">
                <input type="submit" id="user-name-modifying-submit" value="Apply">
            </form>
            &nbsp;<code>&lt;<?php echo $user_id; ?>&gt;</code>
        </div>
        <div id="close-frame">
            ⚠ The socket connection has been closed, try <a href="">reload</a> or <a href="/api/newuser?key=<?php echo $room_key; ?>">connect this room as new user</a>.
        </div>
        <div id="error-frame">
            ⚠ There was an error in socket connection, try <a href="/api/newuser?key=<?php echo $room_key; ?>">connect this room as new user</a>.
        </div>
        <hr>
        <form id="chat-input-form">
            <input type="text" name="chat-input" id="chat-input" required>
            <input type="submit" id="chat-input-submit" value="Send" disabled>
        </form>
        <hr>
    </div>
    <div id="chat-messages">
    </div>
</body>

<script>
    const key = "<?php echo $room_key; ?>";
    const id = "<?php echo $user_id; ?>";
    const input_form = document.forms["chat-input-form"];
    const name_display = document.getElementById("user-name");
    const name_display_frame = document.getElementById("user-name-frame");
    const name_modify_button = document.getElementById("update-user-name");
    const update_name_form = document.forms["user-name-modifying"];
    const submit_button = input_form.elements["chat-input-submit"];
    const close_frame = document.getElementById("close-frame");
    const error_frame = document.getElementById("error-frame");
    const messages_element = document.getElementById("chat-messages");
    const socket = new WebSocket(`/api/room?key=${key}&id=${id}`);
    let user_name = null;

    socket.onmessage = (event) => {
        const message = JSON.parse(event.data);
        switch (message.type) {
            case "new_message":
                const element = document.createElement("p");
                if (message.value.user_name !== null) {
                    element.appendChild(document.createTextNode(`${message.value.user_name} `));
                }
                const id_element = document.createElement("code");
                id_element.appendChild(document.createTextNode(`<${message.value.user_id}>`));
                element.appendChild(id_element);
                element.appendChild(document.createTextNode(`: ${message.value.content}`));
                messages_element.insertAdjacentElement("afterbegin", element);
                break;
            case "your_name":
                user_name = message.value;
                name_display.textContent = user_name;
                break;
            default:
                console.log("unknown message", message);
        }
    };
    socket.onopen = (event) => {
        submit_button.disabled = false;
        console.log("socket open", JSON.stringify(event, null, 4), event);
    };
    socket.onclose = (event) => {
        submit_button.disabled = true;
        close_frame.style.display = "revert";
        console.log("socket close", JSON.stringify(event, null, 4), event);
    };
    socket.onerror = (event) => {
        submit_button.disabled = true;
        error_frame.style.display = "revert";
        console.log("socket error:", JSON.stringify(event, null, 4), event);
    };

    input_form.onsubmit = (event) => {
        event.preventDefault();
        const input_value = document.getElementById("chat-input").value;
        socket.send(JSON.stringify({
            type: "new_message",
            value: input_value,
        }));
        input_form.reset();
    };

    const user_name_modifying = (modifying) => {
        name_display_frame.style.display = modifying ? "none" : "revert";
        update_name_form.style.display = modifying ? "inline" : "none";
    };

    update_name_form.onsubmit = (event) => {
        event.preventDefault();
        user_name  = document.getElementById("user-name-modifying-input").value;
        name_display.textContent = user_name;
        socket.send(JSON.stringify({
            type: "update_name",
            value: user_name,
        }));
        update_name_form.reset();
    };
    update_name_form.onreset = (event) => {
        event.preventDefault();
        document.getElementById("user-name-modifying-input").value = user_name;
        user_name_modifying(false);
    };
    name_modify_button.onclick = (event) => {
        event.preventDefault();
        user_name_modifying(true);
    }
</script>

</html>
