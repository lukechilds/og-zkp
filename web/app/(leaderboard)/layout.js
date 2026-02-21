import SearchInput from '../SearchInput';

export default function LeaderboardLayout({ children }) {
  return (
    <>
      <div className="search-form">
        <SearchInput />
      </div>
      {children}
    </>
  );
}
