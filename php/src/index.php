<!DOCTYPE html>
<html>
    <head>
        <title>The Chat-Room</title>
    </head>
    <body>
        <p>Hi</p>

        <form action="/api/newuser" class="room-key-form">
            <label for="key">Your room key: </label>
            <input type="text" name="key" id="key"
                required minlength="5" maclength="5"
                pattern="^[a-z]{5}$" title="key should only contains lower case letters">
            <input type="submit" value="Connect">
        </form>
    </body>
</html>
