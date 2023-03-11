function copy(event) {
  if (event.isTrusted && event.target == event.currentTarget) {
    const slug = event.target.nextElementSibling.innerText
    navigator.clipboard.writeText(`${window.location.origin}/s/${slug}`)
    event.target.blur()
  }
}
function remove(slug) {
  if (typeof slug == 'string') {
    fetch(`${window.location.origin}/s`, {
      method: "DELETE",
      body: JSON.stringify({ "text": slug }),
      headers: { "Content-Type": "application/json" }
    }).then(res => {
      if (res.status < 300) {
        document.getElementById(slug)?.remove()

        const form = document.querySelector('form')
        form.reset()
        form.lastElementChild.lastElementChild.children.item(1).innerText = ''
        form.lastElementChild.lastElementChild.children.item(2).disabled = true
        form.classList.add('hidden')
      }
    })
  }
}
function edit(event, slug) {
  if (event.isTrusted && event.target == event.currentTarget) {
    event.target.blur()
    const date = event.target.parentElement.previousElementSibling
    const url = date.previousElementSibling
    const slugText = url.previousElementSibling.innerText.trim()

    const form = document.querySelector('form#edit')
    form.querySelector('input[name=slug]').value = slugText
    form.querySelector('input[name=url]').value = url.innerText.trim()

    const from = new Date(+date.getAttribute('data-since'))
    form.querySelector('input[name=fromdate]').value =
      from.getFullYear()
      +'-'+
      ('0'+(from.getMonth()+1)).slice(-2)
      +'-'+
      ('0'+from.getDate()).slice(-2);
    form.querySelector('input[name=fromtime]').value =
      ('0'+from.getHours()).slice(-2)
      +':'+
      ('0'+from.getMinutes()).slice(-2)
      +':'+
      ('0'+from.getSeconds()).slice(-2);

      const to = new Date(+date.getAttribute('data-until'))
      form.querySelector('input[name=todate]').value =
        to.getFullYear()
        +'-'+
        ('0'+(to.getMonth()+1)).slice(-2)
        +'-'+
        ('0'+to.getDate()).slice(-2);
      form.querySelector('input[name=totime]').value =
        ('0'+to.getHours()).slice(-2)
        +':'+
        ('0'+to.getMinutes()).slice(-2)
        +':'+
        ('0'+to.getSeconds()).slice(-2);

    const deleteButton = form.firstElementChild.lastElementChild.lastElementChild
    if (deleteButton instanceof HTMLButtonElement) {
      deleteButton.addEventListener('click', () => remove(slugText), { once: true })
      deleteButton.disabled = false
    }

    form.classList.remove('hidden')
    form.querySelector('input')?.focus()
  }
}
function create(event) {
  if (event.isTrusted && event.target == event.currentTarget) {
    const form = document.querySelector('form#edit')
    form.classList.remove('hidden')
    form.querySelector('input')?.focus()
  }
}
function hide(event) {
  const form = event.currentTarget
  if (event.target == event.currentTarget && event.isTrusted && form instanceof HTMLFormElement) {
    form.reset()

    form.lastElementChild.lastElementChild.children.item(1).innerText = ''
    form.lastElementChild.lastElementChild.children.item(2).disabled = true

    form.classList.add('hidden')
  }
}
function put(event) {
  try {
    if (event.isTrusted && event.currentTarget == event.target && event.target instanceof HTMLFormElement) {
      let form = Object.fromEntries(new FormData(event.target))
      if (form.slug.length == 0 || form.slug.length > 64) {
        throw new Error('Slug creation: please provide a slug (max. 64).')
      }
      form.url = new URL(form.url).toString()
      form.fromdate = form.fromdate ? form.fromdate.toString() : new Date().toISOString().split("T")[0]
      form.fromtime = form.fromtime ? form.fromtime.toString() : '00:00'
      form.todate   = form.todate ? form.todate.toString() : "9999-01-01"
      form.totime   = form.totime ? form.totime.toString() : '00:00'

      const since = new Date(form.fromdate+'T'+form.fromtime).getTime()
      const until = new Date(form.todate+'T'+form.totime).getTime()

      fetch(`${window.location.origin}/s`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          slug: form.slug,
          url: form.url,
          since,
          until,
        }),
      })
      .then((res) => {
        if (res.status < 300) {
          const now = Date.now()
          const gate = now < since ? 'countdown' : now > until ? 'blocked' : 'reachable'
          const row = document.createElement('tr')
          row.id = form.slug
          row.innerHTML = `<td class="px-4 py-2 border border-offblack2">
            <button
              class="group/copy transition-colors hover:text-blue-500 focus:outline focus:outline-1 active:outline-none focus:outline-solid"
              onclick="copy(event)">
              <svg class="group-active/copy:scale-75 pointer-events-none" fill="currentColor" focusable="false"
                aria-hidden="true" viewBox="0 0 24 24" height="20" width="20">
                <path
                  d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z">
                </path>
              </svg>
            </button>
            <span class="truncate">${form.slug}</span>
          </td>
          <td class="px-4 py-2 border border-offblack2 truncate hidden md:table-cell">${form.url}</td>
          <td class="px-4 py-2 border border-offblack2 truncate hidden md:table-cell" data-since="${since}"
            data-until="${until}">${gate}</td>
          <td class="px-4 py-2 border border-offblack2">
            <button
              class="group/edit transition-colors hover:text-blue-500 px-1 focus:outline focus:outline-1 active:outline-none focus:outline-solid"
              onclick="edit(event, '${form.slug}')">
              <span class="group-active/edit:scale-75 block pointer-events-none">edit</span>
            </button>
          </td>`

          const current = document.getElementById(form.slug)
          if (current == null) {
            document.querySelector('table').appendChild(row)
          } else {
            current.replaceWith(row)
          }

          event.target.reset()
          event.target.lastElementChild.lastElementChild.children.item(1).innerText = ''
          event.target.classList.add('hidden')
        } else {
          res.text().then(message =>
            event.target.lastElementChild.lastElementChild.children.item(1).innerText = message
          )
        }
      })
    }
  } catch (error) {
    if (error instanceof Error) {
      event.target.lastElementChild.lastElementChild.children.item(1).innerText = error.message
    }
  }
  return false
}
