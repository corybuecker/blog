import AnalyticsCollector from '@corybuecker/analytics-collector'

AnalyticsCollector.initialize('https://analytics.corybuecker.com', 'blog').start()

const localizeTimeElements = () => {
  const timeElements: HTMLCollectionOf<HTMLTimeElement> = document.getElementsByTagName('time')
  for (const el of timeElements) {
    const timeString = el.dateTime
    el.innerText = new Date(timeString).toLocaleDateString()
  }
}

window.addEventListener('DOMContentLoaded', () => {
  localizeTimeElements()
})
