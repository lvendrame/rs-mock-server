select
  c.id as carrier_id,
  c.name as carrier,
  c.mode as mode,
  count(*) as shipments,
  count(distinct s.warehouse_id) as warehouses
from warehouse_carriers c
join warehouse_shipments s on s.carrier_id = c.id
group by c.id, c.name, c.mode
having count(*) >= 1
order by shipments desc, carrier asc
