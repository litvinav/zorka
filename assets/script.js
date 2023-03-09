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
    const url = event.target.parentElement.previousElementSibling
    const slugText = url.previousElementSibling.innerText.trim()

    const form = document.querySelector('form#popup')
    form.querySelector('input[name=slug]').value = slugText
    form.querySelector('input[name=url]').value = url.innerText.trim()

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
    const form = document.querySelector('form#popup')
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
  const form = event.currentTarget
  try {
    if (event.isTrusted && event.currentTarget == event.target && form instanceof HTMLFormElement) {
      let { slug, url } = Object.fromEntries(new FormData(form))
      if (slug.length == 0 || slug.length > 64) {
        throw new Error('Slug creation: please provide a slug (max. 64).')
      }
      url = new URL(url).toString()
      fetch(`${window.location.origin}/s`, {
        method: "PUT",
        body: JSON.stringify({ slug, url }),
        headers: { "Content-Type": "application/json" }
      })
      .then((res) => {
        if (res.status < 300) {
          const row = document.createElement('tr')
          row.id = slug
          row.innerHTML = `<td class="px-4 py-2 border border-offblack2">
        <button class="group/copy transition-colors hover:text-blue-500 focus:outline focus:outline-1 active:outline-none focus:outline-solid" onclick="copy(event)">
          <svg class="group-active/copy:scale-75 pointer-events-none" fill="currentColor" focusable="false" aria-hidden="true"
            viewBox="0 0 24 24" height="20" width="20">
            <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z">
            </path>
          </svg>
        </button>
        <span class="truncate">${slug}</span>
      </td>
      <td class="px-4 py-2 border border-offblack2">${url}</td>
      <td class="px-4 py-2 border border-offblack2">
        <button class="group/edit transition-colors hover:text-blue-500 px-1 focus:outline focus:outline-1 active:outline-none focus:outline-solid" onclick="edit(event, '${slug}')">
          <span class="group-active/edit:scale-75 block pointer-events-none">edit</span>
        </button>
      </td>`

          const current = document.getElementById(slug)
          if (current == null) {
            document.querySelector('table').appendChild(row)
          } else {
            current.replaceWith(row)
          }

          form.reset()
          form.lastElementChild.lastElementChild.children.item(1).innerText = ''
          form.classList.add('hidden')
        } else {
          res.text().then(message => {
            event.target.lastElementChild.lastElementChild.children.item(1).innerText = message
          })
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
