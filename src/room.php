<?php
$room_key = $_GET["key"] ?? null;
if (is_null($room_key) || preg_match("/^[a-z]{5}$/", $room_key) === 0) {
    header("Location: /");
} elseif (preg_match("/^\/room\.php.*/", $_SERVER['REQUEST_URI']) === 1) {
    header("Location: /room/" . $room_key);
}
?>

<!DOCTYPE html>
<html>
    <head>
        <title>Room</title>
    </head>
    <body>
        <p>Hi, here is the room: <?php echo $room_key ?>.</p>
    </body>
</html>
