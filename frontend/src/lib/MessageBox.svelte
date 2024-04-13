<script lang="ts">
  let message = ""
  
  function post() {
    console.log(message)
    fetch("/text/test", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        "text": message
      })
      
    })
  }
</script>

<form>
  <input type="text" bind:value={message} />
  <input type="button" value="Send Message" on:click={post} />
</form>
