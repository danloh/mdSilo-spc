export const getFavicon = (url: string) => {
  const hostname = url ? new URL(url).hostname : "";
  return "https://icons.duckduckgo.com/ip3/" + hostname + ".ico";
};

export function fmtDatetime(date: string | number | Date) {
  const dt = typeof date === "number" ? date * 1000 : date;
  return new Date(dt).toLocaleString(undefined, {
    weekday: 'short',
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function dateCompare(d1: string | Date, d2: string | Date) {
  return new Date(d1).getTime() - new Date(d2).getTime();
}
