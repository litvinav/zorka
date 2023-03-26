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
      body: JSON.stringify({ "slug": slug }),
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
function edit(event) {
  if (event.isTrusted && event.target == event.currentTarget) {
    event.target.blur()
    const gate = event.target.parentElement.previousElementSibling
    const url = gate.previousElementSibling
    const slugText = url.previousElementSibling.innerText.trim()

    const form = document.querySelector('form#edit')
    form.querySelector('input[name=slug]').value = slugText
    form.querySelector('input[name=url]').value = url.innerText.trim()

    const from = new Date(+gate.getAttribute('data-since'))
    form.querySelector('input[name=fromdate]').value =
      from.getFullYear()
      +'-'+
      ('0'+(from.getMonth()+1)).slice(-2)
      +'-'+
      ('0'+from.getDate()).slice(-2)
    form.querySelector('input[name=fromtime]').value =
      ('0'+from.getHours()).slice(-2)
      +':'+
      ('0'+from.getMinutes()).slice(-2)
      +':'+
      ('0'+from.getSeconds()).slice(-2)

    const to = new Date(+gate.getAttribute('data-until'))
    form.querySelector('input[name=todate]').value =
      to.getFullYear()
      +'-'+
      ('0'+(to.getMonth()+1)).slice(-2)
      +'-'+
      ('0'+to.getDate()).slice(-2)
    form.querySelector('input[name=totime]').value =
      ('0'+to.getHours()).slice(-2)
      +':'+
      ('0'+to.getMinutes()).slice(-2)
      +':'+
      ('0'+to.getSeconds()).slice(-2)

    form.querySelector('input[name=approval]').checked = gate.getAttribute('data-trust') == "untrusted"

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

const svgs = {
  reachable: '',
  countdown: `<svg fill="#ffd700" class="inline-block" fill="currentColor" focusable="false" aria-hidden="true"
  viewBox="0 0 24 24" height="24" width="24" title="Countdown">
  <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zm3.3 14.71L11 12.41V7h2v4.59l3.71 3.71-1.42 1.41z"></path>
</svg>`,
  blocker: `<svg fill="#e14148" class="inline-block" focusable="false" aria-hidden="true"
  viewBox="0 0 24 24" height="24" width="24" title="Blocker">
  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11H7v-2h10v2z"></path>
</svg>`,
  trusted: '',
  untrusted: `<svg fill="#4169e1" class="inline-block" focusable="false" aria-hidden="true" viewBox="0 0 24 24" height="24" width="24" title="Approval">
  <path d="M12 2 4 5v6.09c0 5.05 3.41 9.76 8 10.91 4.59-1.15 8-5.86 8-10.91V5l-8-3zm-1.06 13.54L7.4 12l1.41-1.41 2.12 2.12 4.24-4.24 1.41 1.41-5.64 5.66z"></path>
</svg>`
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
      form.approval = form.approval == "on"

      const since = new Date(form.fromdate+'T'+form.fromtime).getTime()
      const until = new Date(form.todate+'T'+form.totime).getTime()

      fetch(`${window.location.origin}/s`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          slug: form.slug,
          url: form.url,
          approval: form.approval,
          since,
          until,
        }),
      })
      .then((res) => {
        if (res.status < 300) {
          const now = Date.now()
          const gate = now < since ? svgs['countdown'] : now > until ? svgs['blocked'] : svgs['reachable']
          const trust = form.approval ? svgs['untrusted'] : svgs['trusted']
          const row = document.createElement('tr')
          row.id = form.slug
          row.innerHTML = `<td class="px-4 py-2 border border-offblack2 truncate">
          <button class="group/copy transition-colors hover:text-star-dark focus:outline focus:outline-1 active:outline-none focus:outline-solid"
            onclick="copy(event)">
            <svg class="group-active/copy:scale-75 inline-block pointer-events-none" fill="currentColor" focusable="false"
              aria-hidden="true" viewBox="0 0 24 24" height="20" width="20">
              <path
                d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z">
              </path>
            </svg>
          </button>
          <span>${form.slug}</span>
        </td>
        <td class="px-4 py-2 border border-offblack2 truncate hidden md:table-cell">${form.url}</td>
        <td class="px-4 py-2 border border-offblack2 truncate" data-trust="${form.approval?'untrusted':'trusted'}" data-since="${since}"
          data-until="${until}">
          ${gate} ${trust}
        </td>
        <td class="px-4 py-2 border border-offblack2">
          <button
            class="transition-colors hover:text-star-dark focus:outline focus:outline-1 active:outline-none focus:outline-solid"
            onclick="edit(event)">
              <svg class="inline-block pointer-events-none" fill="currentColor" focusable="false" aria-hidden="true"  viewBox="0 0 24 24" height="24" width="24"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"></path></svg>
          </button>
          <a target="_blank" class="active:scale-75 transition-colors hover:text-star-dark focus:outline focus:outline-1 active:outline-none focus:outline-solid" href="/share/${form.slug}">
            <svg class="inline-block pointer-events-none" fill="currentColor" focusable="false" aria-hidden="true" viewBox="0 0 24 24" height="24" width="24"><path d="M15 21h-2v-2h2v2zm-2-7h-2v5h2v-5zm8-2h-2v4h2v-4zm-2-2h-2v2h2v-2zM7 12H5v2h2v-2zm-2-2H3v2h2v-2zm7-5h2V3h-2v2zm-7.5-.5v3h3v-3h-3zM9 9H3V3h6v6zm-4.5 7.5v3h3v-3h-3zM9 21H3v-6h6v6zm7.5-16.5v3h3v-3h-3zM21 9h-6V3h6v6zm-2 10v-3h-4v2h2v3h4v-2h-2zm-2-7h-4v2h4v-2zm-4-2H7v2h2v2h2v-2h2v-2zm1-1V7h-2V5h-2v4h4zM6.75 5.25h-1.5v1.5h1.5v-1.5zm0 12h-1.5v1.5h1.5v-1.5zm12-12h-1.5v1.5h1.5v-1.5z"></path></svg>
          </a>
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
