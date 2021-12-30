using System;
using space_data;
using UnityEngine;
using game;
using game.ui;
using utils;
using System.Collections;
using System.Collections.Generic;

/**
 * Provide domain logic
 */
namespace game.domain
{
    /// used to serialize a uint
    [System.Serializable]
    public class Id
    {
        public ulong value;

        public Id()
        {
            value = 0;
        }

        public Id(ulong value)
        {
            this.value = value;
        }
    }

    public struct V2D
    {
        public float x;
        public float y;

        public V2D(float x, float y)
        {
            this.x = x;
            this.y = y;
        }
    }

    /// Hold all game logic in Unity3d
    public class Domain : MonoBehaviour
    {
        public Transform root;
        public GenericObject prefabGeneric;
        public GenericObject prefabSector;
        public GenericObject prefabJump;
        public GenericObject prefabAsteroid;
        public GenericObject prefabStation;
        private Dictionary<ulong, GameObject> idMap = new();
        private List<GenericObject> sectors = new List<GenericObject>();

        public UIController ui;

        void Start()
        {
        }

        public List<GenericObject> ListSectors()
        {
            return sectors;
        }

        public List<ulong> ListObjectsAtSector(ulong sectorId)
        {
            return new List<ulong>();
        }

        Vector3 GetSectorPos(V2D pos)
        {
            var scale = 15f;
            return new Vector3(pos.x * scale, pos.y * scale, 0f);
        }

        public void AddJump(uint id, uint fromSectorId, V2D fromPos, uint toSectorId, V2D toPos)
        {
            Debug.Log("AddJump " + id + ", " + fromSectorId + "(" + fromPos + ") => " + toSectorId + "(" + toPos + ")");

            var obj = Utils.Inst(prefabJump);
            obj.id = new Id(id);
            obj.kind = ObjKind.Jump;
            obj.UpdateName();

            SetAt(obj.gameObject, fromSectorId);
            obj.transform.localPosition = ToV3(fromPos);

            this.idMap.Add(id, obj.gameObject);

            // update arrow
            {
                var target = this.idMap[toSectorId];
                if (target == null)
                {
                    Debug.LogWarning($"sector {toSectorId} game object not found");
                }
                else
                {
                    {
                        var arrow = obj.GetComponent<DrawGizmosArrow>();
                        arrow.fromPos = obj.transform.position;
                        arrow.toPos = target.transform.position + ToV3(toPos);
                    }
                }

                ui.Refresh();
            }
        }

        public void AddObj(uint id, EntityKind kind)
            {
                Debug.Log("AddObj " + id + "/" + kind);

                GenericObject prefab;

                switch (kind)
                {
                    case EntityKind.Asteroid:
                        prefab = this.prefabAsteroid;
                        break;
                    case EntityKind.Station:
                        prefab = this.prefabAsteroid;
                        break;
                    default:
                        prefab = this.prefabGeneric;
                        break;
                }

                ;

                var obj = Utils.Inst(prefab);
                obj.id = new Id(id);
                obj.kind = (ObjKind) (short) kind;
                obj.UpdateName();
                obj.Hide();

                Utils.SetParentZero(obj.transform, root);

                this.idMap.Add(id, obj.gameObject);
            }

            public void AddSector(ulong id, V2D pos)
            {
                var posV3 = GetSectorPos(pos);

                Debug.Log("AddSector " + id + " at " + posV3);

                var obj = Utils.Inst(prefabSector);
                obj.id = new Id(id);
                obj.kind = ObjKind.Sector;
                obj.UpdateName();
                Utils.SetParentZero(obj.transform, root);

                obj.transform.position = posV3;

                this.idMap.Add(id, obj.gameObject);

                this.sectors.Add(obj);
            }

            public void ObjDock(uint id, uint targetId)
            {
                // Debug.Log("ObjDock " + id + " at " + targetId);
                var obj = this.idMap[id];
                obj.GetComponent<GenericObject>().Hide();
                obj.transform.localPosition = Vector3.zero;
                SetAt(obj, targetId);
            }

            public void ObjJump(uint id, uint sectorId, V2D pos)
            {
                // Debug.Log("ObjJump " + id + " to " + sectorId + " " + pos);
                var obj = this.idMap[id];
                SetAt(obj, sectorId);
                obj.transform.localPosition = new Vector3((float) pos.x, (float) pos.y, 0f);
            }

            public void ObjMove(uint id, V2D pos)
            {
                // Debug.Log("ObjMove");
                var obj = this.idMap[id];
                obj.transform.localPosition = new Vector3((float) pos.x, (float) pos.y, 0f);
            }

            public void ObjTeleport(uint id, uint sectorId, V2D pos)
            {
                // Debug.Log("ObjTeleport " + id + " " + sectorId + "/" + pos.x + ", " + pos.y);
                var obj = this.idMap[id];
                SetAt(obj, sectorId);
                obj.transform.localPosition = new Vector3((float) pos.x, (float) pos.y, 0f);
                obj.GetComponent<GenericObject>().Show();
            }

            public void ObjUndock(uint id, uint sectorId, V2D pos)
            {
                // Debug.Log("ObjUndock " + id + " " + sectorId + "/" + pos.x + ", " + pos.y);
                var obj = this.idMap[id];
                obj.GetComponent<GenericObject>().Show();
                obj.transform.localPosition = new Vector3((float) pos.x, (float) pos.y, 0f);
                SetAt(obj, sectorId);
            }

            ///  Local position is preserve
            private void SetAt(GameObject obj, uint parentId)
            {
                var parent = this.idMap[parentId];
                var localPos = obj.transform.localPosition;
                obj.transform.parent = parent.transform;
                obj.transform.localPosition = localPos;
            }

            private Vector3 ToV3(V2D pos)
            {
                return new Vector3(pos.x, pos.y, 0f);
            }
        }
    }