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
        #close-frame {
            display: none;
        }
        #error-frame {
            display: none;
        }
    </style>
</head>

<body>
    <div class="top">
        <a href="/"><p>&lt; go home</p></a>
        <h1>Room "<?php echo $room_key; ?>"</h1>
        <p>user id: <?php echo $user_id; ?></p>
        <div id="close-frame">
            The socket connection has been closed try <a href="">reload</a> or <a href="/api/newuser?key=<?php echo $room_key; ?>">connect this room as new user</a>.
        </div>
        <div id="error-frame">
            There was an error in socket connection, try <a href="/api/newuser?key=<?php echo $room_key; ?>">connect this room as new user</a>.
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
    const submit_button = input_form.elements["chat-input-submit"];
    const close_frame = document.getElementById("close-frame");
    const error_frame = document.getElementById("error-frame");
    const messages_element = document.getElementById("chat-messages");
    const socket = new WebSocket(`/api/room?key=${key}&id=${id}`);

    socket.onmessage = (event) => {
        const message = event.data;
        const element = document.createElement("p");
        const content = document.createTextNode(message);
        element.appendChild(content);
        messages_element.insertAdjacentElement("afterbegin", element);
    };
    socket.onerror = (event) => {
        submit_button.disabled = true;
        error_frame.style.display = "revert";
        console.log("socket error:", JSON.stringify(event, null, 4), event);
    };
    socket.onclose = (event) => {
        submit_button.disabled = true;
        close_frame.style.display = "revert";
        console.log("socket close", JSON.stringify(event, null, 4), event);
    };
    socket.onopen = (event) => {
        submit_button.disabled = false;
        console.log("socket open", JSON.stringify(event, null, 4), event);
    };

    input_form.onsubmit = (event) => {
        event.preventDefault();
        const input_value = document.getElementById("chat-input").value;
        socket.send(input_value);
        input_form.reset();
    };
</script>

</html>
