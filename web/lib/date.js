export function formatMonth(ts) {
  const n = parseInt(ts, 10);
  if (isNaN(n)) return ts;
  const d = new Date(n * 1000);
  const months = ['January','February','March','April','May','June','July','August','September','October','November','December'];
  return months[d.getUTCMonth()] + ' ' + d.getUTCFullYear();
}

export function getRank(ts) {
  const n = parseInt(ts, 10);
  if (isNaN(n)) return '';
  const year = new Date(n * 1000).getUTCFullYear();
  if (year <= 2009) return 'Legend';
  if (year <= 2011) return 'Cypherpunk';
  if (year <= 2013) return 'Pioneer';
  if (year <= 2015) return 'Veteran';
  if (year <= 2017) return 'Hodler';
  if (year <= 2019) return 'Stacker';
  return 'Pleb';
}

export function formatAge(ts) {
  const n = parseInt(ts, 10);
  if (isNaN(n)) return '';
  const then = new Date(n * 1000);
  const now = new Date();
  let years = now.getUTCFullYear() - then.getUTCFullYear();
  let months = now.getUTCMonth() - then.getUTCMonth();
  if (months < 0) { years--; months += 12; }
  if (years > 0 && months > 0) return `${years}y ${months}m`;
  if (years > 0) return `${years}y`;
  if (months > 0) return `${months}m`;
  return '<1m';
}
