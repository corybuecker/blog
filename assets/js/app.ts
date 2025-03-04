const localizeTimeElements = () => {
  const timeElements: HTMLCollectionOf<HTMLTimeElement> = document.getElementsByTagName('time')
  for (const el of timeElements) {
    const timeString = el.dateTime
    el.innerText = new Date(timeString).toLocaleDateString()
  }
}

localizeTimeElements()

const recordPageview = (): undefined => {
  const analyticsUrl = new URL('https://exlytics.corybuecker.com')
  const pageUrl = new URL(window.location.toString())
  const data = {
    path: pageUrl.pathname,
    host: pageUrl.host,
    event: "page:view",
    method: "GET"
  }

  navigator.sendBeacon(analyticsUrl.toString(), JSON.stringify(data))
  return
}

recordPageview()

const trackAnchor = (element: HTMLAnchorElement): undefined => {
  element.addEventListener('click', event => {
    const analyticsUrl = new URL('https://exlytics.corybuecker.com')
    const href = event.target.getAttribute("href")
    let linkUrl
    if (href.startsWith("/")) {
      linkUrl = URL.parse(`${window.location.host}${href}`)
    } else {
      linkUrl = URL.parse(href)
    }
    const data = {
      path: linkUrl.pathname,
      host: linkUrl.host,
      event: "link:click",
      method: "GET"
    }
    console.log(data)
    navigator.sendBeacon(analyticsUrl.toString(), JSON.stringify(data))
    return
  })

  return
}

const anchorElements = document.getElementsByTagName('a')
Array.from(anchorElements).forEach(trackAnchor)
