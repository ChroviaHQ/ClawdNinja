export function Logo() {
  return (
    <svg
      className='h-4 w-auto shrink-0'
      viewBox='2.6 3.6 12.4 8.4'
      shapeRendering='crispEdges'
      aria-hidden='true'
    >
      <g fill='#D97757'>
        <rect x='4' y='4.5' width='8' height='5' />
        <rect x='3' y='6.5' width='10' height='2' />
        <rect x='4.5' y='9.5' width='1' height='2' />
        <rect x='6.25' y='9.5' width='1' height='2' />
        <rect x='8.75' y='9.5' width='1' height='2' />
        <rect x='10.5' y='9.5' width='1' height='2' />
      </g>
      <g className='fill-blindfold'>
        <rect x='3.75' y='5.6' width='8.5' height='1.3' />
        <g className='logo-tail'>
          <rect x='12.25' y='5.4' width='.75' height='1.2' />
          <rect x='13' y='4.8' width='.8' height='.8' />
          <rect x='13.8' y='4.1' width='.8' height='.8' />
          <rect x='13' y='5.7' width='.9' height='.7' />
          <rect x='13.9' y='5.5' width='.8' height='.7' />
        </g>
      </g>
    </svg>
  );
}
