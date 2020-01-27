# NTFS to Netex France conversion

Version of the implementation: `1.0`

## Introduction

This document describes how a [NTFS feed](https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md) is transformed into a Netex profil France feed in Navitia Transit Model.

The resulting ZIP archive is composed of:
- a `arrets.xml` file containing the description of all stops (Quays and StopPlaces)
- a `correspondances.xml` file containing all transfers between stops
- a folder for each network containing
  + TBD

## Schema Validation
To validate the produced XML document, you can use `xmllint` tool.  On a
Debian/Ubuntu environment, install the `libxml2-utils` package.

```shell
apt install libxml2-utils
```

On Windows, you can follow [these
instructions](https://stackoverflow.com/questions/19546854/installing-xmllint).

For validation, you need NeTEx schemas.

```shell
git clone https://github.com/NeTEx-CEN/NeTEx.git
```

Then you can validate your file `my_file.xml` against NeTEx schema with the
following command.

```shell
xmllint --noout --nonet --huge --schema /path/to/NeTEx/xsd/NeTEx_publication.xsd my_file.xml
```

## Input parameters

A NTFS feed is not sufficient to generate a valid Netex Profil France feed:
- `ParticipantRef` : The ParticipantRef code must be provided,
- `StopProviderCode` : The code used to identify the provider in the stop_id generation.

## File headers

Each Netex file starts with a header containing some information generated by the program: 
- `/PublicationDelivery/@version`: **x.y:FR-NETEX_nnnn-a.b-c** with
  + `x.y`:  the version of Netex XSD of Netex (currently 1.09)
  + `nnnn`:identification of the profile ("FRANCE", "COMMUN", "ARRET",
"LIGNE", "RESEAU", "HORAIRE", "CALENDRIER" or "TARIF") 
  + `a.b` is the profile version (2.1)
  + `c` is the number of the local implementation (containing only integers and `.`). See at the top of the document.
- `/PublicationDelivery/PublicationTimestamp`: generation date of the Netex Feed using ISO8601,
- `/PublicationDelivery/ParticipantRef`: The value of the corresponding parameter to the conversion action.

## arrets.xml

A `stop_area` is considered monomodal if all the trips having stop_times referencing any of its stop_points have a physical_mode of the same "Netex mode".

### Quay

Each `Quay` node corresponds to a NTFS `stop_point`.

Netex field | NTFS file | NTFS field | Note
--- | --- | --- | ---
Quay/@id | stops.txt | stop_id | see (1) below.
Quay/@version | | | fixed value `any`.
Quay/PublicCode | stops.txt | stop_code | This node may not be prensent if the stop_point has no `stop_code`.
Quay/Name | stops.txt | stop_name | 
Quay/Centroid/Location | stops.txt | stop_lat and stop_lon | see (2) below
SiteRef | stops.txt | parent_station | If the parent_station is multimodal, see (3) below.
TransportMode | | | see (4) below
tariffZones | stops.txt | fare_zone_id | The fare zone is prefixed by the `ParticipantRef` prefix with a `:` separator
AccessibilityAssessment | stops.txt | equipment_id | This node is present only if the `equipment_id` is specified. see [`AccessibilityAssessment`](#accessibilityassessment) below.

**(1) definition of id**
The id is composed of several parts separated by `:`: 
- a country code (ISO 3166-1). `FR` for France.
- a city code (5 character INSEE code for France). Eventually, the city disctrict can be specified with one or two character separated by `-`. For exemple `75056-12`.
- object type using an enumarate:
  + `ZE` (ZONE D’EMBARQUEMENT), 
  + `LMO` (LIEU D’ARRET MONOMODAL), 
  + `PM` (POLE MONOMODAL), 
  + `LMU` (LIEU D’ARRET MUTIMODAL), 
  + `AC` (ACCES)
- technical code of the stop
- code of the provider of the technical stop or `LOC` if it's beeing attributed. 

In this version of the connector:
- the country code will be set to the fixed value `FR`,
- the city code will be set to the fixed value `XXXXX`,
- the technical code of the stop will contain the value of `stop_id`, with a replacement of `:` by `_` to avoid conflict between separators,
- the code of the provider will be set to the value of `StopProviderCode` param (is specified) or  `LOC` otherwise.


**(2) definition of Location**

The GTFS stop_lon and stop_lat are specified in WGS84. The coordinates are converted to EPSG:2154 (Lambert 93).  
Example of Netex declaration:
><gml:pos srsName="EPSG:2154">662233.0 6861519.0</gml:pos>

**(3) definition of SiteRef**

TBD

**(4) definition of the TransportMode**

As a stop_point can be associated to several physical_modes, all the physical_modes need to be mapped to the Netex list.
If more than one Netex mode is associated, the most frequent one is used and a warning is emitted.

physical_mode_id | TransportMode in Netex
--- | --- 
Air | air
Boat | water
Bus | bus
BusRapidTransit | bus
Coach | coach
Ferry | water
Funicular | funicular
LocalTrain | rail
LongDistanceTrain | rail
Metro | metro
RapidTransit | rail
RailShuttle | rail
Shuttle | bus
SuspendedCableCar | cableway
Taxi | _ignored_
Train | train
Tramway | tram

#### AccessibilityAssessment

If the stop_point is associated to an equipment, a node `AccessibilityAssessment` is created and its content is as follow:

Netex field | NTFS file | NTFS field | Note
--- | --- | --- | ---
AccessibilityAssessment/@id | stops.txt | equipment_id | 
AccessibilityAssessment/MobilityImpairedAccess | | | see (1) below
AccessibilityAssessment/ limitations/AccessibilityLimitation/ WheelchairAccess | equipments.txt | wheelchair_boarding | see (2) below
AccessibilityAssessment/ limitations/AccessibilityLimitation/ AudibleSignsAvailable | equipments.txt | audible_announcement | see (2) below
AccessibilityAssessment/ limitations/AccessibilityLimitation/ VisualSignsAvailable | equipments.txt | visual_announcement | see (2) below

**(1) definition of MobilityImpairedAccess**

As stated in `NF_Profil NeTEx éléments communs(F) - v2.1.pdf` in chapter 5.10:
`AccessibilityAssessment` is optional, but if it's present, `MobilityImpairedAccess` is mandatory. Therefore, `MobilityImpairedAccess` sould be set to:
- `true` if all `AccessibilityLimitation` are set to `true`
- `false` if all `AccessibilityLimitation` are set to `false`
- `partial` if some `AccessibilityLimitation` are set to `true`
- `unknow` in other cases

**(2) value of limitations**

NTFS accessibility value | Netex accessibility value
--- | ---
0 or undefined | `undefined` 
1 | `true`
2 | `false`

### monomodalStopPlace and multimodalStopPlace

warnings: 
- in monomodalStopPlace: all stop_points should have the same name. If not, 2 StopPlace must be created,
- multimodalStopPlace: show the higher rated mode value only
