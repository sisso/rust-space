using System;
using space_data;
using UnityEngine;
using game;
using utils;
using System.Collections;
using System.Collections.Generic;

/**
 * Provide domain logic
 */
namespace game.domain
{
    [System.Serializable]
    public class Id
    {
        public uint value;

        public Id()
        {
            value = 0;
        }

        public Id(uint value)
        {
            this.value = value;
        }
    }

    public class Domain : MonoBehaviour, core.EventHandler
    {
        public Transform root;
        public GenericObject prefabGeneric;
        public Dictionary<uint, GameObject> idMap = new Dictionary<uint, GameObject>();

        public void AddJump(uint id, uint fromSectorId, V2 fromPos, uint toSectorId, V2 toPos)
        {
            Debug.Log("AddJump");
        }

        public void AddObj(uint id, EntityKind kind)
        {
            Debug.Log("AddObj " + id + "/" + kind);

            var obj = Utils.Inst(prefabGeneric);
            obj.id = new Id(id);
            obj.kind = (ObjKind)(short)kind;

            Utils.SetParentZero(obj.transform, root);

            this.idMap.Add(id, obj.gameObject);
        }

        public void AddSector(uint id)
        {
            Debug.Log("AddSector " + id);

            var obj = Utils.Inst(prefabGeneric);
            obj.id = new Id(id);
            obj.kind = ObjKind.Sector;
            Utils.SetParentZero(obj.transform, root);

            obj.transform.position = new Vector3((float) id * 10f, 0.0f, 0.0f);

            this.idMap.Add(id, obj.gameObject);
        }

        public void ObjDock(uint id, uint targetId)
        {
            Debug.Log("ObjDock");
            var obj = this.idMap[id];
            obj.GetComponent<DrawGizmos>().enabled = false;
        }

        public void ObjJump(uint id, uint sectorId, V2 pos)
        {
            Debug.Log("ObjJump");
            var obj = this.idMap[id];
            obj.transform.position = new Vector3((float)pos.X, (float)pos.Y, 0f);
        }

        public void ObjMove(uint id, V2 pos)
        {
            // Debug.Log("ObjMove");
            var obj = this.idMap[id];
            obj.transform.position = new Vector3((float)pos.X, (float)pos.Y, 0f);
        }

        public void ObjTeleport(uint id, uint sectorId, V2 pos)
        {
            Debug.Log("ObjTeleport " + id + " " + sectorId + "/" +pos.X + ", " +pos.Y);
            var obj = this.idMap[id];
            obj.transform.position = new Vector3((float)pos.X, (float)pos.Y, 0f);
        }

        public void ObjUndock(uint id, uint sectorId, V2 pos)
        {
            Debug.Log("ObjUndock " + id + " " + sectorId + "/" + pos.X + ", " + pos.Y);
            var obj = this.idMap[id];
            obj.GetComponent<DrawGizmos>().enabled = true;
            obj.transform.position = new Vector3((float)pos.X, (float)pos.Y, 0f);
        }
    }
}
