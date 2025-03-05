<!DOCTYPE html>
<html>
    <head>
        <title>Example</title>
    </head>
    <body>
        <p>Hi</p>

        <form action="/room.php" class="room-key-form">
          <div class="room-key-form">
            <label for="key">Your room key: </label>
            <input type="text" name="key" id="key"
                required minlength="5" maclength="5"
                pattern="^[a-z]{5}$" title="key should only contains lower case letters">
          </div>
          <div class="room-key-form">
            <input type="submit" value="Connect">
          </div>
        </form>
    </body>
</html>
